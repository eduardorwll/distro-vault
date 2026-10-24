use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

use domain::entities::package::PackageSource;
use domain::errors::DomainError;
use domain::ports::{AgentPort, BrowserPort, PackagePort, SshPort, StoragePort, SystemPort};
use domain::value_objects::{DistroFamily, PackageName};

/// DTO for requesting a full snapshot restoration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub archive_path: PathBuf,
    pub target_config_root: PathBuf,
    pub target_home: PathBuf,
    pub target_custom_base: Option<PathBuf>,
    pub install_packages: bool,
    pub dry_run: bool,
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
            install_packages: false,
            dry_run: false,
        }
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
    package_ports: Vec<Arc<dyn PackagePort>>,
}

impl RestorePipelineUseCase {
    pub fn new(
        browser_port: Arc<dyn BrowserPort>,
        storage_port: Arc<dyn StoragePort>,
        agent_port: Arc<dyn AgentPort>,
        system_port: Arc<dyn SystemPort>,
        ssh_port: Arc<dyn SshPort>,
        package_ports: Vec<Arc<dyn PackagePort>>,
    ) -> Self {
        Self {
            browser_port,
            storage_port,
            agent_port,
            system_port,
            ssh_port,
            package_ports,
        }
    }

    pub fn execute(&self, req: RestoreRequest) -> Result<RestoreReport, DomainError> {
        // 1. Read manifest from archive stream
        let manifest = self.storage_port.read_manifest(&req.archive_path)?;

        // 2. Extract snapshot into isolated staging area
        let staging_dir = tempdir().map_err(|e| DomainError::ManifestError {
            message: format!("Failed to create staging directory for restoration: {}", e),
        })?;

        self.storage_port.extract_snapshot(&req.archive_path, staging_dir.path())?;

        // 3. Restore Browser Profiles & Normalize Hashes
        let mut profiles_restored = Vec::new();
        let staging_browser = staging_dir.path().join("browser");
        if staging_browser.exists() {
            for prof in &manifest.zen_profiles {
                let prof_staging_path = staging_browser.join(prof.id.as_str());
                if prof_staging_path.exists() {
                    if !req.dry_run {
                        self.browser_port.restore_profile_state(
                            &staging_browser,
                            &req.target_config_root,
                        )?;
                        self.browser_port.normalize_installation_hash(
                            &req.target_config_root,
                            &prof.id,
                        )?;
                    }
                    profiles_restored.push(prof.id.as_str().to_string());
                }
            }
        }

        // 4. Restore Agent Governance
        let staging_agents = staging_dir.path().join("agents");
        let skills_count = manifest.agent_governance.skills.len();
        if staging_agents.exists() && !req.dry_run {
            self.agent_port.restore_governance(&staging_agents, &req.target_home)?;
        }

        // 5. Detect Target Distro & Package Dispatch
        let target_distro = self.system_port.detect_distro().unwrap_or(DistroFamily::Unknown);
        let is_same_distro = target_distro == manifest.source_distro;

        let mut packages_installed = Vec::new();
        let mut pending_packages = Vec::new();

        if req.install_packages {
            for port in &self.package_ports {
                let port_source = port.source();
                let matching_pkgs: Vec<_> = manifest
                    .packages
                    .iter()
                    .filter(|p| p.source == port_source)
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

        // 6. Check FUSE2 requirement for AppImages
        let fuse2_missing_warning = !manifest.appimages.is_empty() && !self.system_port.is_fuse2_available();

        // 7. Restore Registered SSH Keys with strict POSIX permissions (0700/0600)
        let staging_ssh = staging_dir.path().join("ssh");
        let ssh_keys_restored = if staging_ssh.exists() && !manifest.ssh_keys.is_empty() {
            self.ssh_port.restore_ssh_keys(
                &staging_ssh,
                &req.target_home,
                &manifest.ssh_keys,
                req.dry_run,
            )?
        } else {
            0
        };

        // 8. Restore Custom Directories
        for custom_d in &manifest.custom_directories {
            let custom_staging = staging_dir.path().join(&custom_d.archive_prefix);
            if custom_staging.exists() {
                let target_dest = if let Some(ref base) = req.target_custom_base {
                    base.join(&custom_d.id)
                } else {
                    custom_d.source_path.clone()
                };

                if !req.dry_run {
                    if let Some(parent) = target_dest.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ = copy_dir_recursively(&custom_staging, &target_dest);
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
            packages_installed,
            pending_packages,
            fuse2_missing_warning,
        })
    }
}

fn copy_dir_recursively(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
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
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}
