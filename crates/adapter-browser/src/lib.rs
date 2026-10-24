pub mod mozlz4;
pub mod profile_normalizer;

use std::fs;
use std::path::Path;

use domain::entities::zen_profile::{Container, SessionInfo, Space, ZenProfile};
use domain::errors::DomainError;
use domain::ports::BrowserPort;
use domain::value_objects::ProfileId;

pub use mozlz4::MozLz4Codec;
pub use profile_normalizer::{mozilla_install_hash, ProfilesIniNormalizer};

#[derive(Debug, Default, Clone)]
pub struct ZenBrowserAdapter;

impl ZenBrowserAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Extracts workspaces/spaces and containers from a decompressed zen session JSON
    fn parse_session_metadata(
        json_content: &str,
        compressed_size: usize,
    ) -> (Vec<Space>, Vec<Container>, Option<SessionInfo>) {
        let mut spaces = Vec::new();
        let mut containers = Vec::new();
        let mut windows_count = 0;
        let mut total_tabs = 0;

        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_content) {
            if let Some(win_arr) = v.get("windows").and_then(|w| w.as_array()) {
                windows_count = win_arr.len();
                for win in win_arr {
                    if let Some(tabs) = win.get("tabs").and_then(|t| t.as_array()) {
                        total_tabs += tabs.len();
                    }
                }
            }

            // Check Zen-specific spaces/workspaces
            if let Some(space_arr) = v.pointer("/spaces").and_then(|s| s.as_array()) {
                for s in space_arr {
                    let id = s
                        .get("id")
                        .and_then(|i| i.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let name = s
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Workspace")
                        .to_string();
                    let icon = s
                        .get("icon")
                        .and_then(|ic| ic.as_str())
                        .map(|s| s.to_string());
                    let color = s
                        .get("color")
                        .and_then(|co| co.as_str())
                        .map(|s| s.to_string());
                    let container_id = s
                        .get("container_id")
                        .and_then(|ci| ci.as_str())
                        .map(|s| s.to_string());
                    let tab_count = s
                        .get("tabs")
                        .and_then(|t| t.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    if !id.is_empty() {
                        spaces.push(Space {
                            id,
                            name,
                            icon,
                            color,
                            container_id,
                            open_tabs_count: tab_count,
                            pinned_tabs_count: 0,
                        });
                    }
                }
            }

            // Check contextual identities (Containers)
            if let Some(container_arr) = v.pointer("/containers").and_then(|c| c.as_array()) {
                for c in container_arr {
                    let id = c
                        .get("userContextId")
                        .and_then(|i| i.as_u64())
                        .map(|u| u.to_string())
                        .or_else(|| c.get("id").and_then(|i| i.as_str()).map(|s| s.to_string()))
                        .unwrap_or_default();
                    let name = c
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Container")
                        .to_string();
                    let color = c
                        .get("color")
                        .and_then(|co| co.as_str())
                        .unwrap_or("blue")
                        .to_string();
                    let icon = c
                        .get("icon")
                        .and_then(|ic| ic.as_str())
                        .unwrap_or("fingerprint")
                        .to_string();
                    if !id.is_empty() {
                        containers.push(Container {
                            id,
                            name,
                            color,
                            icon,
                        });
                    }
                }
            }
        }

        let session_info = Some(SessionInfo {
            uncompressed_size: json_content.len(),
            compressed_size,
            windows_count,
            spaces_count: spaces.len(),
            total_tabs,
        });

        (spaces, containers, session_info)
    }

    /// Recursively copy directory contents skipping lock files and ephemeral caches
    fn copy_profile_recursively(src: &Path, dst: &Path) -> Result<(), DomainError> {
        if !dst.exists() {
            fs::create_dir_all(dst).map_err(|e| DomainError::BrowserError {
                message: format!("Failed to create destination directory {:?}: {}", dst, e),
            })?;
        }

        let entries = fs::read_dir(src).map_err(|e| DomainError::BrowserError {
            message: format!("Failed to read source directory {:?}: {}", src, e),
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            // Skip volatile caches, locks and sockets
            if name_str == ".parentlock"
                || name_str == "lock"
                || name_str == "cache2"
                || name_str == "startupCache"
                || name_str == "thumbnails"
            {
                continue;
            }

            let dest_path = dst.join(&file_name);
            if path.is_dir() {
                Self::copy_profile_recursively(&path, &dest_path)?;
            } else if path.is_file() {
                fs::copy(&path, &dest_path).map_err(|e| DomainError::BrowserError {
                    message: format!("Failed to copy file {:?} to {:?}: {}", path, dest_path, e),
                })?;
            }
        }

        Ok(())
    }
}

impl BrowserPort for ZenBrowserAdapter {
    fn is_locked(&self, profile_root: &Path) -> Result<bool, DomainError> {
        let lock_file = profile_root.join(".parentlock");
        Ok(lock_file.exists())
    }

    fn scan_profiles(&self, config_root: &Path) -> Result<Vec<ZenProfile>, DomainError> {
        if !config_root.exists() {
            return Ok(Vec::new());
        }

        let mut profiles = Vec::new();
        let entries = fs::read_dir(config_root).map_err(|e| DomainError::BrowserError {
            message: format!("Failed to read config root {:?}: {}", config_root, e),
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dir_name = entry.file_name().to_string_lossy().to_string();

            // Look for SQLite or session files characteristic of a Firefox/Zen profile
            let session_file = path.join("zen-sessions.jsonlz4");
            let fallback_session = path.join("sessionstore.jsonlz4");
            let places_file = path.join("places.sqlite");

            if session_file.exists() || fallback_session.exists() || places_file.exists() {
                let profile_id = ProfileId::new(dir_name.clone())?;
                // Default is true if it contains "Default" or "release" in directory name
                let is_default = dir_name.to_lowercase().contains("default");
                let mut zen_prof = ZenProfile::new(profile_id, is_default, path.clone());

                let active_session = if session_file.exists() {
                    Some(session_file)
                } else if fallback_session.exists() {
                    Some(fallback_session)
                } else {
                    None
                };

                if let Some(sess_path) = active_session {
                    if let Ok(bytes) = fs::read(&sess_path) {
                        if let Ok(json_str) = MozLz4Codec::decompress(&bytes) {
                            let (spaces, containers, s_info) =
                                Self::parse_session_metadata(&json_str, bytes.len());
                            zen_prof.spaces = spaces;
                            zen_prof.containers = containers;
                            zen_prof.session_info = s_info;
                        }
                    }
                }

                zen_prof.has_bookmarks = places_file.exists();
                zen_prof.has_passwords =
                    path.join("logins.json").exists() || path.join("key4.db").exists();
                if path.join("extensions.json").exists() {
                    zen_prof.extensions_count = 1;
                }

                profiles.push(zen_prof);
            }
        }

        Ok(profiles)
    }

    fn decompress_session(&self, compressed_bytes: &[u8]) -> Result<String, DomainError> {
        MozLz4Codec::decompress(compressed_bytes)
    }

    fn compress_session(&self, json_payload: &str) -> Result<Vec<u8>, DomainError> {
        MozLz4Codec::compress(json_payload)
    }

    fn normalize_installation_hash(
        &self,
        config_root: &Path,
        target_profile_id: &ProfileId,
    ) -> Result<(), DomainError> {
        ProfilesIniNormalizer::normalize(config_root, target_profile_id.as_str())
    }

    fn export_profile_state(
        &self,
        profile: &ZenProfile,
        dest_dir: &Path,
    ) -> Result<(), DomainError> {
        if self.is_locked(&profile.root_path)? {
            return Err(DomainError::BrowserLocked {
                lock_file: profile.root_path.join(".parentlock"),
            });
        }

        let target_profile_dest = dest_dir.join(profile.id.as_str());
        Self::copy_profile_recursively(&profile.root_path, &target_profile_dest)
    }

    fn restore_profile_state(
        &self,
        source_dir: &Path,
        config_root: &Path,
    ) -> Result<(), DomainError> {
        let entries = fs::read_dir(source_dir).map_err(|e| DomainError::BrowserError {
            message: format!("Failed to read source_dir {:?}: {}", source_dir, e),
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                let dest_path = config_root.join(&dir_name);
                Self::copy_profile_recursively(&path, &dest_path)?;

                // Normalize installs.ini and profiles.ini for this profile
                if let Ok(profile_id) = ProfileId::new(&dir_name) {
                    let _ = self.normalize_installation_hash(config_root, &profile_id);
                }
            }
        }

        Ok(())
    }
}
