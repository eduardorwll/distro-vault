use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::{tempdir, tempdir_in, TempDir};

use domain::entities::package::PackageSource;
use domain::entities::selection_filter::SelectionFilter;
use domain::errors::DomainError;
use domain::ports::{
    AgentPort, BrowserPort, GitConfigPort, PackagePort, SshPort, StoragePort, SystemPort,
};
use domain::value_objects::{DistroFamily, PackageName};

/// DTO for requesting a full snapshot restoration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub archive_path: PathBuf,
    pub target_config_root: PathBuf,
    pub target_home: PathBuf,
    pub target_custom_base: Option<PathBuf>,
    pub staging_override: Option<PathBuf>,
    pub install_packages: bool,
    pub dry_run: bool,
    pub selection_filter: Option<SelectionFilter>,
}

impl RestoreRequest {
    pub fn new(
        archive_path: impl Into<PathBuf>,
        target_config_root: impl Into<PathBuf>,
        target_home: impl Into<PathBuf>,
    ) -> Self {
        Self {
            archive_path: archive_path.into(),
            target_config_root: target_config_root.into(),
            target_home: target_home.into(),
            target_custom_base: None,
            staging_override: None,
            install_packages: false,
            dry_run: false,
            selection_filter: None,
        }
    }

    pub fn with_filter(mut self, filter: SelectionFilter) -> Self {
        self.selection_filter = Some(filter);
        self
    }

    pub fn with_packages(mut self, install: bool, dry_run: bool) -> Self {
        self.install_packages = install;
        self.dry_run = dry_run;
        self
    }

    pub fn with_custom_base(mut self, base: impl Into<PathBuf>) -> Self {
        self.target_custom_base = Some(base.into());
        self
    }

    pub fn with_staging_override(mut self, staging: impl Into<PathBuf>) -> Self {
        self.staging_override = Some(staging.into());
        self
    }
}

/// DTO summarizing the outcome of a restoration operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreReport {
    pub snapshot_id: String,
    pub source_distro: DistroFamily,
    pub target_distro: DistroFamily,
    pub profiles_restored: Vec<String>,
    pub skills_restored: usize,
    pub ssh_keys_restored: usize,
    pub git_configs_restored: usize,
    pub packages_installed: Vec<PackageName>,
    pub pending_packages: Vec<PackageName>,
    pub fuse2_missing_warning: bool,
}

pub struct RestorePipelineUseCase {
    browser_port: Arc<dyn BrowserPort>,
    storage_port: Arc<dyn StoragePort>,
    agent_port: Arc<dyn AgentPort>,
    system_port: Arc<dyn SystemPort>,
    ssh_port: Arc<dyn SshPort>,
    git_config_port: Arc<dyn GitConfigPort>,
    package_ports: Vec<Arc<dyn PackagePort>>,
}

impl RestorePipelineUseCase {
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

    pub fn execute(&self, req: RestoreRequest) -> Result<RestoreReport, DomainError> {
        // 1. Read manifest from staging or archive stream
        let manifest = if let Some(ref override_path) = req.staging_override {
            let manifest_file = override_path.join("manifest.json");
            if manifest_file.exists() {
                let file = fs::File::open(&manifest_file).map_err(|e| DomainError::ManifestError {
                    message: format!("Failed to open manifest in staging: {}", e),
                })?;
                serde_json::from_reader(file).map_err(|e| DomainError::ManifestError {
                    message: format!("Failed to parse manifest in staging: {}", e),
                })?
            } else {
                self.storage_port.read_manifest(&req.archive_path)?
            }
        } else {
            self.storage_port.read_manifest(&req.archive_path)?
        };

        // 2. Extract snapshot into isolated staging area (or use existing staging override)
        let _staging_guard: Option<TempDir>;
        let staging_path: &Path = if let Some(ref override_path) = req.staging_override {
            _staging_guard = None;
            override_path.as_path()
        } else {
            let dir = create_staging_dir(&req.target_home, &req.archive_path).map_err(|e| {
                DomainError::ManifestError {
                    message: format!("Failed to create staging directory for restoration: {}", e),
                }
            })?;
            self.storage_port
                .extract_snapshot(&req.archive_path, dir.path())?;
            _staging_guard = Some(dir);
            _staging_guard
                .as_ref()
                .map(|d| d.path())
                .ok_or_else(|| DomainError::ManifestError {
                    message: "Unexpected failure retrieving staging directory path".to_string(),
                })?
        };

        // 3. Restore Browser Profiles & Normalize Hashes
        let mut profiles_restored = Vec::new();
        let should_restore_zen = req
            .selection_filter
            .as_ref()
            .map(|f| f.restore_zen)
            .unwrap_or(true);
        let staging_browser = staging_path.join("browser");
        if staging_browser.exists() && should_restore_zen {
            for prof in &manifest.zen_profiles {
                if let Some(ref filter) = req.selection_filter {
                    if !filter.is_space_included(prof.id.as_str()) {
                        continue;
                    }
                }
                let prof_staging_path = staging_browser.join(prof.id.as_str());
                if prof_staging_path.exists() {
                    if !req.dry_run {
                        self.browser_port
                            .restore_profile_state(&staging_browser, &req.target_config_root)?;
                        self.browser_port
                            .normalize_installation_hash(&req.target_config_root, &prof.id)?;
                    }
                    profiles_restored.push(prof.id.as_str().to_string());
                }
            }
        }

        // 4. Restore Agent Governance
        let staging_agents = staging_path.join("agents");
        let mut skills_count = manifest.agent_governance.skills.len();
        if let Some(ref filter) = req.selection_filter {
            skills_count = manifest
                .agent_governance
                .skills
                .iter()
                .filter(|s| filter.is_skill_included(&s.name))
                .count();
        }
        if staging_agents.exists() && !req.dry_run && skills_count > 0 {
            self.agent_port
                .restore_governance(&staging_agents, &req.target_home)?;
        }

        // 5. Restore Registered SSH Keys with strict POSIX permissions (0700/0600)
        let staging_ssh = staging_path.join("ssh");
        let mut target_ssh_keys = manifest.ssh_keys.clone();
        if let Some(ref filter) = req.selection_filter {
            target_ssh_keys.retain(|k| filter.is_ssh_key_included(&k.file_name));
        }
        let ssh_keys_restored = if staging_ssh.exists() && !target_ssh_keys.is_empty() {
            self.ssh_port.restore_ssh_keys(
                &staging_ssh,
                &req.target_home,
                &target_ssh_keys,
                req.dry_run,
            )?
        } else {
            0
        };

        // 6. Restore Global Git Configuration (.gitconfig, .config/git/config)
        let staging_git = staging_path.join("git");
        let should_restore_git = req
            .selection_filter
            .as_ref()
            .map(|f| f.restore_git_config)
            .unwrap_or(true);
        let git_configs_restored = if staging_git.exists() && should_restore_git && !manifest.git_configs.is_empty() {
            self.git_config_port.restore_git_config(
                &staging_git,
                &req.target_home,
                &manifest.git_configs,
                req.dry_run,
            )?
        } else {
            0
        };

        // 7. Restore Custom Directories with Path Transcription to target home
        for custom_d in &manifest.custom_directories {
            if let Some(ref filter) = req.selection_filter {
                if !filter.is_custom_dir_included(&custom_d.id) {
                    continue;
                }
            }
            let custom_staging = staging_path.join(&custom_d.archive_prefix);
            if custom_staging.exists() {
                let target_dest = if let Some(ref base) = req.target_custom_base {
                    base.join(&custom_d.id)
                } else {
                    transcribe_path_to_target_home(&custom_d.source_path, &req.target_home)
                };

                if !req.dry_run {
                    if let Some(parent) = target_dest.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    copy_dir_recursively(&custom_staging, &target_dest).map_err(|e| {
                        DomainError::ManifestError {
                            message: format!(
                                "Failed to restore custom directory {:?} to {:?}: {}",
                                custom_d.id, target_dest, e
                            ),
                        }
                    })?;
                }
            }
        }

        // 8. Check FUSE2 requirement for AppImages
        let mut appimages_to_check = manifest.appimages.clone();
        if let Some(ref filter) = req.selection_filter {
            appimages_to_check.retain(|a| filter.is_appimage_included(&a.name));
        }
        let fuse2_missing_warning =
            !appimages_to_check.is_empty() && !self.system_port.is_fuse2_available();

        // 9. Detect Target Distro & Package Dispatch
        let target_distro = self
            .system_port
            .detect_distro()
            .unwrap_or(DistroFamily::Unknown);
        let is_same_distro = target_distro == manifest.source_distro;

        let mut packages_installed = Vec::new();
        let mut pending_packages = Vec::new();

        if req.install_packages {
            for port in &self.package_ports {
                let port_source = port.source();
                let matching_pkgs: Vec<_> = manifest
                    .packages
                    .iter()
                    .filter(|p| {
                        p.source == port_source
                            && req
                                .selection_filter
                                .as_ref()
                                .map(|f| f.is_package_included(&p.name))
                                .unwrap_or(true)
                    })
                    .cloned()
                    .collect();

                if matching_pkgs.is_empty() {
                    continue;
                }

                if is_same_distro || port_source == PackageSource::Flatpak {
                    if let Ok(installed) = port.install_batch(&matching_pkgs, req.dry_run) {
                        packages_installed.extend(installed);
                    }
                } else {
                    // Distro-hop: foreign native packages queued as pending
                    for p in matching_pkgs {
                        pending_packages.push(p.name);
                    }
                }
            }
        }

        Ok(RestoreReport {
            snapshot_id: manifest.snapshot_id,
            source_distro: manifest.source_distro,
            target_distro,
            profiles_restored,
            skills_restored: skills_count,
            ssh_keys_restored,
            git_configs_restored,
            packages_installed,
            pending_packages,
            fuse2_missing_warning,
        })
    }
}

/// Transcribes an absolute path from the source distribution/user to the target user home directory.
/// If the path was inside a user home (e.g. `/home/<source_user>/...`), it re-roots it under `target_home`.
pub fn transcribe_path_to_target_home(original: &Path, target_home: &Path) -> PathBuf {
    let components: Vec<_> = original.components().collect();
    if components.len() >= 3 {
        if let (
            std::path::Component::RootDir,
            std::path::Component::Normal(home_dir),
            std::path::Component::Normal(_user),
        ) = (components[0], components[1], components[2])
        {
            if home_dir == "home" {
                let mut relative = PathBuf::new();
                for comp in &components[3..] {
                    relative.push(comp);
                }
                return target_home.join(relative);
            }
        }
    }

    if original.is_relative() {
        target_home.join(original)
    } else {
        original.to_path_buf()
    }
}

fn create_staging_dir(target_home: &Path, archive_path: &Path) -> std::io::Result<TempDir> {
    let user_cache = target_home.join(".cache");
    if user_cache.is_dir() {
        if let Ok(dir) = tempdir_in(&user_cache) {
            return Ok(dir);
        }
    }

    if let Some(archive_parent) = archive_path.parent() {
        if archive_parent.is_dir() {
            if let Ok(dir) = tempdir_in(archive_parent) {
                return Ok(dir);
            }
        }
    }

    if target_home.is_dir() {
        if let Ok(dir) = tempdir_in(target_home) {
            return Ok(dir);
        }
    }

    tempdir()
}

fn copy_dir_recursively(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursively(&path, &dest)?;
        } else {
            if dest.exists() || dest.is_symlink() {
                let _ = fs::remove_file(&dest);
            }
            if fs::rename(&path, &dest).is_err() {
                fs::copy(&path, &dest)?;
            }
        }
    }
    Ok(())
}
