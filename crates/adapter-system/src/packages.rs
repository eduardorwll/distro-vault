use std::process::Command;

use domain::entities::package::{PackageCategory, PackageScope, PackageSource};
use domain::entities::PackageEntry;
use domain::errors::DomainError;
use domain::ports::PackagePort;
use domain::value_objects::PackageName;

/// Adapter for Pacman explicit native packages
#[derive(Debug, Default, Clone)]
pub struct PacmanAdapter;

impl PacmanAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PackagePort for PacmanAdapter {
    fn source(&self) -> PackageSource {
        PackageSource::PacmanNative
    }

    fn query_installed(&self) -> Result<Vec<PackageEntry>, DomainError> {
        let output = Command::new("pacman")
            .args(["-Qne"])
            .output()
            .map_err(|e| DomainError::PackageError {
                message: format!("Failed to run pacman -Qne: {}", e),
            })?;

        if !output.status.success() {
            return Err(DomainError::PackageError {
                message: format!(
                    "pacman -Qne exited with error: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        let mut list = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&name_str) = parts.first() {
                if let Ok(pkg_name) = PackageName::new(name_str) {
                    let (scope, category) = PackageCategory::classify(name_str, PackageSource::PacmanNative);
                    let mut entry = PackageEntry::new(pkg_name, PackageSource::PacmanNative, scope)
                        .with_category(category);
                    if let Some(&ver) = parts.get(1) {
                        entry = entry.with_version(ver);
                    }
                    list.push(entry);
                }
            }
        }

        Ok(list)
    }

    fn install_batch(
        &self,
        packages: &[PackageEntry],
        dry_run: bool,
    ) -> Result<Vec<PackageName>, DomainError> {
        let names: Vec<PackageName> = packages.iter().map(|p| p.name.clone()).collect();
        if dry_run || names.is_empty() {
            return Ok(names);
        }

        let name_strs: Vec<&str> = names.iter().map(|n| n.as_str()).collect();
        let status = Command::new("sudo")
            .arg("pacman")
            .args(["-S", "--needed", "--noconfirm"])
            .args(&name_strs)
            .status()
            .map_err(|e| DomainError::PackageError {
                message: format!("Failed to execute pacman install: {}", e),
            })?;

        if !status.success() {
            return Err(DomainError::PackageError {
                message: "Pacman batch installation failed".to_string(),
            });
        }

        Ok(names)
    }

    fn is_available(&self, name: &PackageName) -> Result<bool, DomainError> {
        let status = Command::new("pacman")
            .args(["-Si", name.as_str()])
            .output();

        match status {
            Ok(out) => Ok(out.status.success()),
            Err(_) => Ok(false),
        }
    }

    fn query_dependencies(&self) -> Result<Vec<PackageEntry>, DomainError> {
        let output = Command::new("pacman")
            .args(["-Qnd"])
            .output()
            .map_err(|e| DomainError::PackageError {
                message: format!("Failed to run pacman -Qnd: {}", e),
            })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let mut list = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&name_str) = parts.first() {
                if let Ok(pkg_name) = PackageName::new(name_str) {
                    let category = PackageCategory::classify_dependency(name_str);
                    let mut entry = PackageEntry::new(pkg_name, PackageSource::PacmanNative, PackageScope::System)
                        .with_category(category)
                        .as_dependency(true);
                    if let Some(&ver) = parts.get(1) {
                        entry = entry.with_version(ver);
                    }
                    list.push(entry);
                }
            }
        }

        Ok(list)
    }
}

/// Adapter for AUR packages (paru or yay)
#[derive(Debug, Default, Clone)]
pub struct AurAdapter;

impl AurAdapter {
    pub fn new() -> Self {
        Self
    }

    fn aur_helper() -> &'static str {
        if which::which("paru").is_ok() {
            "paru"
        } else {
            "yay"
        }
    }
}

mod which {
    use std::path::Path;

    pub fn which(cmd: &str) -> Result<(), ()> {
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in path_var.split(':') {
                if Path::new(dir).join(cmd).is_file() {
                    return Ok(());
                }
            }
        }
        Err(())
    }
}

impl PackagePort for AurAdapter {
    fn source(&self) -> PackageSource {
        PackageSource::Aur
    }

    fn query_installed(&self) -> Result<Vec<PackageEntry>, DomainError> {
        let helper = Self::aur_helper();
        let output = Command::new(helper)
            .args(["-Qme"])
            .output()
            .map_err(|e| DomainError::PackageError {
                message: format!("Failed to run {} -Qme: {}", helper, e),
            })?;

        if !output.status.success() {
            return Err(DomainError::PackageError {
                message: format!(
                    "{} -Qme exited with error: {}",
                    helper,
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        let mut list = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&name_str) = parts.first() {
                if let Ok(pkg_name) = PackageName::new(name_str) {
                    let (scope, category) = PackageCategory::classify(name_str, PackageSource::Aur);
                    let mut entry = PackageEntry::new(pkg_name, PackageSource::Aur, scope)
                        .with_category(category);
                    if let Some(&ver) = parts.get(1) {
                        entry = entry.with_version(ver);
                    }
                    list.push(entry);
                }
            }
        }

        Ok(list)
    }

    fn query_dependencies(&self) -> Result<Vec<PackageEntry>, DomainError> {
        let output = Command::new("pacman")
            .args(["-Qmd"])
            .output()
            .map_err(|e| DomainError::PackageError {
                message: format!("Failed to run pacman -Qmd: {}", e),
            })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let mut list = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&name_str) = parts.first() {
                if let Ok(pkg_name) = PackageName::new(name_str) {
                    let category = PackageCategory::classify_dependency(name_str);
                    let mut entry = PackageEntry::new(pkg_name, PackageSource::Aur, PackageScope::User)
                        .with_category(category)
                        .as_dependency(true);
                    if let Some(&ver) = parts.get(1) {
                        entry = entry.with_version(ver);
                    }
                    list.push(entry);
                }
            }
        }

        Ok(list)
    }

    fn install_batch(
        &self,
        packages: &[PackageEntry],
        dry_run: bool,
    ) -> Result<Vec<PackageName>, DomainError> {
        let names: Vec<PackageName> = packages.iter().map(|p| p.name.clone()).collect();
        if dry_run || names.is_empty() {
            return Ok(names);
        }

        let helper = Self::aur_helper();
        let name_strs: Vec<&str> = names.iter().map(|n| n.as_str()).collect();
        let status = Command::new(helper)
            .args(["-S", "--needed", "--noconfirm"])
            .args(&name_strs)
            .status()
            .map_err(|e| DomainError::PackageError {
                message: format!("Failed to execute {} install: {}", helper, e),
            })?;

        if !status.success() {
            return Err(DomainError::PackageError {
                message: format!("{} batch installation failed", helper),
            });
        }

        Ok(names)
    }

    fn is_available(&self, name: &PackageName) -> Result<bool, DomainError> {
        let helper = Self::aur_helper();
        let status = Command::new(helper)
            .args(["-Si", name.as_str()])
            .output();

        match status {
            Ok(out) => Ok(out.status.success()),
            Err(_) => Ok(false),
        }
    }
}

/// Adapter for Flatpak packages
#[derive(Debug, Default, Clone)]
pub struct FlatpakAdapter;

impl FlatpakAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PackagePort for FlatpakAdapter {
    fn source(&self) -> PackageSource {
        PackageSource::Flatpak
    }

    fn query_installed(&self) -> Result<Vec<PackageEntry>, DomainError> {
        let output = Command::new("flatpak")
            .args(["list", "--app", "--columns=application,version,branch"])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return Ok(Vec::new()),
        };

        let mut list = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&app_id) = parts.first() {
                if let Ok(pkg_name) = PackageName::new(app_id) {
                    let (scope, category) = PackageCategory::classify(app_id, PackageSource::Flatpak);
                    let mut entry = PackageEntry::new(pkg_name, PackageSource::Flatpak, scope)
                        .with_category(category);
                    if let Some(&ver) = parts.get(1) {
                        entry = entry.with_version(ver);
                    }
                    list.push(entry);
                }
            }
        }

        Ok(list)
    }

    fn install_batch(
        &self,
        packages: &[PackageEntry],
        dry_run: bool,
    ) -> Result<Vec<PackageName>, DomainError> {
        let names: Vec<PackageName> = packages.iter().map(|p| p.name.clone()).collect();
        if dry_run || names.is_empty() {
            return Ok(names);
        }

        for pkg in packages {
            let status = Command::new("flatpak")
                .args(["install", "-y", "flathub", pkg.name.as_str()])
                .status()
                .map_err(|e| DomainError::PackageError {
                    message: format!("Failed to install flatpak {}: {}", pkg.name, e),
                })?;

            if !status.success() {
                return Err(DomainError::PackageError {
                    message: format!("Flatpak install failed for {}", pkg.name),
                });
            }
        }

        Ok(names)
    }

    fn is_available(&self, name: &PackageName) -> Result<bool, DomainError> {
        let status = Command::new("flatpak")
            .args(["search", name.as_str()])
            .output();

        match status {
            Ok(out) => Ok(out.status.success() && !out.stdout.is_empty()),
            Err(_) => Ok(false),
        }
    }
}
