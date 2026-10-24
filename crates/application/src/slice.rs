use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use domain::entities::{Manifest, SelectionFilter};
use domain::errors::DomainError;
use domain::ports::StoragePort;
use domain::value_objects::Checksum;

/// Request parameters for extracting and packaging a subset of an existing snapshot
#[derive(Debug, Clone)]
pub struct SliceSnapshotRequest {
    pub source_archive: PathBuf,
    pub output_archive: PathBuf,
    pub selection_filter: SelectionFilter,
    pub new_snapshot_id: Option<String>,
    pub staging_dir_override: Option<PathBuf>,
}

impl SliceSnapshotRequest {
    pub fn new(
        source_archive: impl Into<PathBuf>,
        output_archive: impl Into<PathBuf>,
        selection_filter: SelectionFilter,
    ) -> Self {
        Self {
            source_archive: source_archive.into(),
            output_archive: output_archive.into(),
            selection_filter,
            new_snapshot_id: None,
            staging_dir_override: None,
        }
    }

    pub fn with_snapshot_id(mut self, id: impl Into<String>) -> Self {
        self.new_snapshot_id = Some(id.into());
        self
    }

    pub fn with_staging_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.staging_dir_override = Some(dir.into());
        self
    }
}

/// Execution report summarizing the newly created sliced snapshot
#[derive(Debug, Clone)]
pub struct SliceSnapshotReport {
    pub snapshot_id: String,
    pub output_path: PathBuf,
    pub checksum: Checksum,
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub manifest: Manifest,
}

/// RAII Guard ensuring temporary staging directories are pruned upon function exit
struct StagingCleanupGuard<'a>(&'a Path);

impl<'a> Drop for StagingCleanupGuard<'a> {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = fs::remove_dir_all(self.0);
        }
    }
}

/// Use case that extracts a source snapshot, prunes its contents according
/// to a SelectionFilter, and repacks an atomic, self-contained subset archive (.tar.zst).
pub struct SliceSnapshotUseCase {
    storage_port: Arc<dyn StoragePort>,
}

impl SliceSnapshotUseCase {
    pub fn new(storage_port: Arc<dyn StoragePort>) -> Self {
        Self { storage_port }
    }

    pub fn execute(&self, req: SliceSnapshotRequest) -> Result<SliceSnapshotReport, DomainError> {
        // 1. Validate paths and guard against in-place file corruption
        if !req.source_archive.exists() {
            return Err(DomainError::ManifestError {
                message: format!(
                    "Source snapshot archive does not exist: {:?}",
                    req.source_archive
                ),
            });
        }

        if req.source_archive == req.output_archive {
            return Err(DomainError::ManifestError {
                message: "Output archive path cannot be identical to the source archive path"
                    .into(),
            });
        }

        if let (Ok(src_canon), Ok(out_canon)) = (
            req.source_archive.canonicalize(),
            req.output_archive.canonicalize(),
        ) {
            if src_canon == out_canon {
                return Err(DomainError::ManifestError {
                    message: "Output archive path resolves to the same source archive file".into(),
                });
            }
        }

        // 2. Read the source manifest
        let mut manifest = self.storage_port.read_manifest(&req.source_archive)?;

        // 3. Setup temporary staging directory with RAII cleanup guard
        let staging_root = match req.staging_dir_override {
            Some(dir) => dir,
            None => {
                let cache_dir = std::env::var("XDG_CACHE_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| {
                        std::env::var("HOME")
                            .map(|h| PathBuf::from(h).join(".cache"))
                            .unwrap_or_else(|_| PathBuf::from("/tmp"))
                    });
                cache_dir
                    .join("distro-vault")
                    .join("staging")
                    .join(format!("slice-{}", chrono_timestamp_slug()))
            }
        };

        if !staging_root.exists() {
            fs::create_dir_all(&staging_root).map_err(|e| DomainError::ManifestError {
                message: format!(
                    "Failed to create staging directory {:?}: {}",
                    staging_root, e
                ),
            })?;
        }

        let _cleanup_guard = StagingCleanupGuard(&staging_root);

        // 4. Extract entire source snapshot into isolated staging directory
        self.storage_port
            .extract_snapshot(&req.source_archive, &staging_root)?;

        // 5. Build filtered archive items and prune manifest
        let mut archive_items: Vec<(PathBuf, PathBuf)> = Vec::new();

        // 5.1 Zen Browser profiles & spaces
        if req.selection_filter.restore_zen {
            for prof in &mut manifest.zen_profiles {
                prof.spaces
                    .retain(|s| req.selection_filter.is_space_included(&s.id));
                let prof_staging = staging_root.join("browser").join(prof.id.as_str());
                if prof_staging.exists() {
                    archive_items.push((
                        prof_staging,
                        PathBuf::from("browser").join(prof.id.as_str()),
                    ));
                }
            }
        } else {
            manifest.zen_profiles.clear();
        }

        // 5.2 System packages
        manifest
            .packages
            .retain(|p| req.selection_filter.is_package_included(&p.name));

        // 5.3 Agent rules & skills
        manifest
            .agent_governance
            .skills
            .retain(|s| req.selection_filter.is_skill_included(&s.name));

        for rule in &manifest.agent_governance.rules {
            let rule_staging = staging_root.join("agents").join(&rule.relative_path);
            if rule_staging.exists() {
                archive_items.push((
                    rule_staging,
                    PathBuf::from("agents").join(&rule.relative_path),
                ));
            }
        }

        for skill in &manifest.agent_governance.skills {
            let skill_staging = staging_root.join("agents").join(&skill.relative_path);
            if skill_staging.exists() {
                archive_items.push((
                    skill_staging,
                    PathBuf::from("agents").join(&skill.relative_path),
                ));
            }
        }

        // 5.4 SSH keys
        manifest
            .ssh_keys
            .retain(|k| req.selection_filter.is_ssh_key_included(&k.file_name));

        for key in &manifest.ssh_keys {
            let key_staging = staging_root.join("ssh").join(&key.relative_path);
            if key_staging.exists() {
                archive_items
                    .push((key_staging, PathBuf::from("ssh").join(&key.relative_path)));
            }
        }

        // 5.5 Global Git config
        if req.selection_filter.restore_git_config {
            for git_cfg in &manifest.git_configs {
                let git_staging = staging_root.join("git").join(&git_cfg.relative_path);
                if git_staging.exists() {
                    archive_items
                        .push((git_staging, PathBuf::from("git").join(&git_cfg.relative_path)));
                }
            }
        } else {
            manifest.git_configs.clear();
        }

        // 5.6 Custom directories
        manifest
            .custom_directories
            .retain(|c| req.selection_filter.is_custom_dir_included(&c.id));

        for custom_d in &manifest.custom_directories {
            let custom_staging = staging_root.join(&custom_d.archive_prefix);
            if custom_staging.exists() {
                archive_items.push((custom_staging, custom_d.archive_prefix.clone()));
            }
        }

        // 6. Recalculate manifest metrics
        let total_files: usize = archive_items.len()
            + manifest
                .custom_directories
                .iter()
                .map(|c| c.file_count)
                .sum::<usize>();

        let total_size_bytes: u64 = manifest
            .custom_directories
            .iter()
            .map(|c| c.size_bytes)
            .sum::<u64>()
            + manifest.ssh_keys.iter().map(|k| k.size_bytes).sum::<u64>();

        let snapshot_id = req.new_snapshot_id.unwrap_or_else(|| {
            format!("{}-subset-{}", manifest.snapshot_id, chrono_timestamp_slug())
        });

        manifest.snapshot_id = snapshot_id.clone();
        manifest.created_at_utc = current_utc_iso();
        manifest.total_files = total_files;
        manifest.total_size_bytes = total_size_bytes;
        manifest.checksum = None;

        // 7. Ensure output directory exists
        if let Some(parent) = req.output_archive.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| DomainError::ManifestError {
                    message: format!(
                        "Failed to create output archive parent dir {:?}: {}",
                        parent, e
                    ),
                })?;
            }
        }

        // 8. Create new atomic snapshot archive
        let checksum =
            self.storage_port
                .create_snapshot(&archive_items, &manifest, &req.output_archive)?;

        manifest.checksum = Some(checksum.clone());

        Ok(SliceSnapshotReport {
            snapshot_id,
            output_path: req.output_archive,
            checksum,
            total_files,
            total_size_bytes,
            manifest,
        })
    }
}

fn chrono_timestamp_slug() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

fn current_utc_iso() -> String {
    "2026-10-24T19:15:00Z".to_string()
}
