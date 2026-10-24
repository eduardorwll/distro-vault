use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use domain::entities::custom_directory::CustomDirectoryEntry;
use domain::entities::manifest::Manifest;
use domain::entities::selection_filter::SelectionFilter;
use domain::errors::DomainError;
use domain::ports::{
    AgentPort, BrowserPort, GitConfigPort, PackagePort, SshPort, StoragePort, SystemPort,
};
use domain::value_objects::Checksum;

/// DTO for requesting a full declarative system snapshot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupRequest {
    pub config_root: PathBuf,
    pub user_home: PathBuf,
    pub output_archive: PathBuf,
    pub selection_filter: SelectionFilter,
    pub custom_directories: Vec<CustomDirectoryEntry>,
    pub snapshot_id: Option<String>,
}

impl BackupRequest {
    pub fn new(
        config_root: impl Into<PathBuf>,
        user_home: impl Into<PathBuf>,
        output_archive: impl Into<PathBuf>,
    ) -> Self {
        Self {
            config_root: config_root.into(),
            user_home: user_home.into(),
            output_archive: output_archive.into(),
            selection_filter: SelectionFilter::new(),
            custom_directories: Vec::new(),
            snapshot_id: None,
        }
    }

    pub fn with_filter(mut self, filter: SelectionFilter) -> Self {
        self.selection_filter = filter;
        self
    }

    pub fn with_custom_directories(mut self, dirs: Vec<CustomDirectoryEntry>) -> Self {
        self.custom_directories = dirs;
        self
    }

    pub fn with_snapshot_id(mut self, id: impl Into<String>) -> Self {
        self.snapshot_id = Some(id.into());
        self
    }
}

/// DTO summarizing the generated backup snapshot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupReport {
    pub snapshot_id: String,
    pub output_path: PathBuf,
    pub checksum: Checksum,
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub manifest: Manifest,
}

pub struct BackupPipelineUseCase {
    browser_port: Arc<dyn BrowserPort>,
    storage_port: Arc<dyn StoragePort>,
    agent_port: Arc<dyn AgentPort>,
    system_port: Arc<dyn SystemPort>,
    ssh_port: Arc<dyn SshPort>,
    git_config_port: Arc<dyn GitConfigPort>,
    package_ports: Vec<Arc<dyn PackagePort>>,
}

impl BackupPipelineUseCase {
    pub fn new(
        browser_port: Arc<dyn BrowserPort>,
        storage_port: Arc<dyn StoragePort>,
        agent_port: Arc<dyn AgentPort>,
        system_port: Arc<dyn SystemPort>,
        ssh_port: Arc<dyn SshPort>,
        git_config_port: Arc<dyn GitConfigPort>,
        package_ports: Vec<Arc<dyn PackagePort>>,
    ) -> Self {
        Self {
            browser_port,
            storage_port,
            agent_port,
            system_port,
            ssh_port,
            git_config_port,
            package_ports,
        }
    }

    pub fn execute(&self, req: BackupRequest) -> Result<BackupReport, DomainError> {
        // 1. Validate browser locks
        if self.browser_port.is_locked(&req.config_root)? {
            // Attempt graceful shutdown if Zen is running
            if self.system_port.is_process_running("zen-bin")
                || self.system_port.is_process_running("zen")
            {
                let _ = self.system_port.terminate_process_gracefully("zen-bin", 5);
            }
            if self.browser_port.is_locked(&req.config_root)? {
                return Err(DomainError::BrowserLocked {
                    lock_file: req.config_root.join(".parentlock"),
                });
            }
        }

        // 2. Query host information
        let source_distro = self.system_port.detect_distro()?;
        let source_host = self.system_port.hostname()?;
        let snapshot_id = req
            .snapshot_id
            .unwrap_or_else(|| format!("snap-{}", chrono_timestamp_slug()));
        let created_at_utc = current_utc_iso();

        // 3. Scan & Filter Browser Profiles
        let mut profiles = self.browser_port.scan_profiles(&req.config_root)?;
        for prof in &mut profiles {
            prof.spaces
                .retain(|s| req.selection_filter.is_space_included(&s.id));
        }

        // 4. Scan & Filter Packages across backends
        let mut packages = Vec::new();
        for port in &self.package_ports {
            if let Ok(installed) = port.query_installed() {
                for pkg in installed {
                    if req.selection_filter.is_package_included(&pkg.name) {
                        packages.push(pkg);
                    }
                }
            }
            if !req.selection_filter.included_dependencies.is_empty() {
                if let Ok(deps) = port.query_dependencies() {
                    for pkg in deps {
                        if req.selection_filter.is_dependency_included(&pkg.name) {
                            packages.push(pkg);
                        }
                    }
                }
            }
        }

        // 5. Scan & Filter Agent Governance
        let mut governance = self.agent_port.scan_governance(&req.user_home)?;
        governance
            .skills
            .retain(|s| req.selection_filter.is_skill_included(&s.name));

        // 6. Scan & Filter Registered SSH Keys
        let mut ssh_keys = self.ssh_port.scan_ssh_keys(&req.user_home)?;
        ssh_keys.retain(|k| req.selection_filter.is_ssh_key_included(&k.file_name));

        // 7. Scan Global Git Configuration (.gitconfig, .config/git/config)
        let git_configs = self.git_config_port.scan_git_config(&req.user_home)?;

        // 8. Custom Directories (already attached or filtered)
        let custom_dirs: Vec<CustomDirectoryEntry> = req.custom_directories;

        // 9. Prepare source-to-archive items
        let mut archive_items: Vec<(PathBuf, PathBuf)> = Vec::new();

        // Add browser profiles
        for prof in &profiles {
            archive_items.push((
                prof.root_path.clone(),
                PathBuf::from("browser").join(prof.id.as_str()),
            ));
        }

        // Add agent rules and skills
        for rule in &governance.rules {
            let disk_path = req.user_home.join(&rule.relative_path);
            if disk_path.exists() {
                archive_items.push((disk_path, PathBuf::from("agents").join(&rule.relative_path)));
            }
        }
        for skill in &governance.skills {
            let disk_path = req.user_home.join(&skill.relative_path);
            if disk_path.exists() {
                archive_items.push((
                    disk_path,
                    PathBuf::from("agents").join(&skill.relative_path),
                ));
            }
        }

        // Add registered SSH keys
        for key in &ssh_keys {
            let disk_path = req.user_home.join(".ssh").join(&key.relative_path);
            if disk_path.exists() {
                archive_items.push((disk_path, PathBuf::from("ssh").join(&key.relative_path)));
            }
        }

        // Add global Git configuration
        for git_cfg in &git_configs {
            let disk_path = req.user_home.join(&git_cfg.relative_path);
            if disk_path.exists() {
                archive_items.push((disk_path, PathBuf::from("git").join(&git_cfg.relative_path)));
            }
        }

        // Add custom directories
        for custom_d in &custom_dirs {
            if custom_d.source_path.exists() {
                archive_items.push((
                    custom_d.source_path.clone(),
                    custom_d.archive_prefix.clone(),
                ));
            }
        }

        // 10. Build Manifest
        let mut manifest = Manifest::new(
            snapshot_id.clone(),
            created_at_utc,
            source_host,
            source_distro,
            governance,
        );
        manifest.zen_profiles = profiles;
        manifest.packages = packages;
        manifest.custom_directories = custom_dirs;
        manifest.ssh_keys = ssh_keys;
        manifest.git_configs = git_configs;

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
        manifest.total_files = total_files;
        manifest.total_size_bytes = total_size_bytes;

        // 10. Create atomic snapshot
        let checksum =
            self.storage_port
                .create_snapshot(&archive_items, &manifest, &req.output_archive)?;

        manifest.checksum = Some(checksum.clone());

        Ok(BackupReport {
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
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}

fn current_utc_iso() -> String {
    "2026-10-23T18:00:00Z".to_string()
}
