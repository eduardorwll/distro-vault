use std::fs;
use std::path::Path;

use domain::errors::DomainError;
use domain::value_objects::DistroFamily;

pub struct OsReleaseParser;

impl OsReleaseParser {
    /// Parses an os-release formatted file (e.g. /etc/os-release) into DistroFamily
    pub fn parse_file(path: &Path) -> Result<DistroFamily, DomainError> {
        if !path.exists() {
            return Err(DomainError::UnsupportedDistro {
                distro: format!("os-release file not found at {:?}", path),
            });
        }

        let content = fs::read_to_string(path).map_err(|e| DomainError::UnsupportedDistro {
            distro: format!("Failed to read os-release at {:?}: {}", path, e),
        })?;

        Self::parse_content(&content)
    }

    /// Parses the raw string contents of an os-release file
    pub fn parse_content(content: &str) -> Result<DistroFamily, DomainError> {
        let mut id = String::new();
        let mut id_like = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("ID=") {
                id = rest.trim_matches('"').trim_matches('\'').to_string();
            } else if let Some(rest) = trimmed.strip_prefix("ID_LIKE=") {
                id_like = Some(rest.trim_matches('"').trim_matches('\'').to_string());
            }
        }

        if id.is_empty() {
            return Err(DomainError::UnsupportedDistro {
                distro: "No ID field found in os-release".to_string(),
            });
        }

        Ok(DistroFamily::from_id(&id, id_like.as_deref()))
    }

    /// Checks if libfuse2 is present on the system for AppImage execution
    pub fn check_fuse2_installed() -> bool {
        let candidate_paths = [
            "/usr/lib/libfuse.so.2",
            "/usr/lib64/libfuse.so.2",
            "/lib/x86_64-linux-gnu/libfuse.so.2",
            "/usr/lib/x86_64-linux-gnu/libfuse.so.2",
            "/lib64/libfuse.so.2",
            "/lib/libfuse.so.2",
        ];

        for p in candidate_paths {
            if Path::new(p).exists() {
                return true;
            }
        }

        // Fallback: check ldconfig output if available
        if let Ok(output) = std::process::Command::new("ldconfig").arg("-p").output() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            if out_str.contains("libfuse.so.2") {
                return true;
            }
        }

        false
    }

    /// Reads system hostname
    pub fn read_hostname() -> Result<String, DomainError> {
        let etc_hostname = Path::new("/etc/hostname");
        if etc_hostname.exists() {
            if let Ok(content) = fs::read_to_string(etc_hostname) {
                let name = content.trim();
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }

        if let Ok(output) = std::process::Command::new("uname").arg("-n").output() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() {
                return Ok(name);
            }
        }

        Ok("localhost".to_string())
    }
}
