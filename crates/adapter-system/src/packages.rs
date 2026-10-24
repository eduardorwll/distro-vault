use std::process::Command;

use domain::entities::package::{PackageCategory, PackageScope, PackageSource};
use domain::entities::PackageEntry;
use domain::errors::DomainError;
use domain::ports::PackagePort;
use domain::value_objects::PackageName;

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

fn is_distro_preinstalled_on_system(name: &str) -> bool {
    if PackageCategory::is_distro_preinstalled(name) {
        return true;
    }
    static DYNAMIC_DISTRO_PKGS: std::sync::OnceLock<std::collections::HashSet<String>> =
        std::sync::OnceLock::new();
    let set = DYNAMIC_DISTRO_PKGS.get_or_init(|| {
        let mut set = std::collections::HashSet::new();
        for file in &[
            "/usr/share/omarchy/install/omarchy-base.packages",
            "/usr/share/omarchy/install/omarchy-other.packages",
            "/etc/calamares/modules/packages.conf",
        ] {
            if let Ok(content) = std::fs::read_to_string(file) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        set.insert(trimmed.to_lowercase());
                    }
                }
            }
        }
        set
    });
    set.contains(&name.to_lowercase())
}

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
        if which::which("pacman").is_err() {
            return Ok(Vec::new());
        }

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
                    let (mut scope, mut category) =
                        PackageCategory::classify(name_str, PackageSource::PacmanNative);
                    if is_distro_preinstalled_on_system(name_str) && scope == PackageScope::User {
                        scope = PackageScope::System;
                        category = PackageCategory::DistroBase;
                    }
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
        let status = Command::new("pacman").args(["-Si", name.as_str()]).output();

        match status {
            Ok(out) => Ok(out.status.success()),
            Err(_) => Ok(false),
        }
    }

    fn query_dependencies(&self) -> Result<Vec<PackageEntry>, DomainError> {
        if which::which("pacman").is_err() {
            return Ok(Vec::new());
        }

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
                    let mut entry = PackageEntry::new(
                        pkg_name,
                        PackageSource::PacmanNative,
                        PackageScope::System,
                    )
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

    fn aur_helper() -> Option<&'static str> {
        if which::which("paru").is_ok() {
            Some("paru")
        } else if which::which("yay").is_ok() {
            Some("yay")
        } else {
            None
        }
    }
}

impl PackagePort for AurAdapter {
    fn source(&self) -> PackageSource {
        PackageSource::Aur
    }

    fn query_installed(&self) -> Result<Vec<PackageEntry>, DomainError> {
        let helper = match Self::aur_helper() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let output = Command::new(helper).args(["-Qme"]).output().map_err(|e| {
            DomainError::PackageError {
                message: format!("Failed to run {} -Qme: {}", helper, e),
            }
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
                    let (mut scope, mut category) =
                        PackageCategory::classify(name_str, PackageSource::Aur);
                    if is_distro_preinstalled_on_system(name_str) && scope == PackageScope::User {
                        scope = PackageScope::System;
                        category = PackageCategory::DistroBase;
                    }
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
        if which::which("pacman").is_err() {
            return Ok(Vec::new());
        }

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
                    let mut entry =
                        PackageEntry::new(pkg_name, PackageSource::Aur, PackageScope::User)
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

        let helper = match Self::aur_helper() {
            Some(h) => h,
            None => {
                return Err(DomainError::PackageError {
                    message: "No AUR helper (paru/yay) available on system".to_string(),
                })
            }
        };
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
        let helper = match Self::aur_helper() {
            Some(h) => h,
            None => return Ok(false),
        };
        let status = Command::new(helper).args(["-Si", name.as_str()]).output();

        match status {
            Ok(out) => Ok(out.status.success()),
            Err(_) => Ok(false),
        }
    }
}

/// Adapter for DNF / RPM native packages (Fedora, RHEL, CentOS)
#[derive(Debug, Default, Clone)]
pub struct DnfAdapter;

impl DnfAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PackagePort for DnfAdapter {
    fn source(&self) -> PackageSource {
        PackageSource::DnfNative
    }

    fn query_installed(&self) -> Result<Vec<PackageEntry>, DomainError> {
        if which::which("dnf").is_err()
            && which::which("dnf5").is_err()
            && which::which("rpm").is_err()
        {
            return Ok(Vec::new());
        }

        let output = if which::which("dnf5").is_ok() {
            Command::new("dnf5")
                .args([
                    "repoquery",
                    "--userinstalled",
                    "--qf",
                    "%{name}\t%{version}-%{release}",
                ])
                .output()
        } else if which::which("dnf").is_ok() {
            Command::new("dnf")
                .args([
                    "repoquery",
                    "--userinstalled",
                    "--qf",
                    "%{name}\t%{version}-%{release}",
                ])
                .output()
        } else {
            Command::new("rpm")
                .args(["-qa", "--qf", "%{NAME}\t%{VERSION}-%{RELEASE}\n"])
                .output()
        };

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return Ok(Vec::new()),
        };

        let mut list = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&name_str) = parts.first() {
                if let Ok(pkg_name) = PackageName::new(name_str) {
                    let (mut scope, mut category) =
                        PackageCategory::classify(name_str, PackageSource::DnfNative);
                    if is_distro_preinstalled_on_system(name_str) && scope == PackageScope::User {
                        scope = PackageScope::System;
                        category = PackageCategory::DistroBase;
                    }
                    let mut entry = PackageEntry::new(pkg_name, PackageSource::DnfNative, scope)
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
        if which::which("dnf").is_err() && which::which("dnf5").is_err() {
            return Ok(Vec::new());
        }

        let cmd = if which::which("dnf5").is_ok() {
            "dnf5"
        } else {
            "dnf"
        };
        let output = Command::new(cmd)
            .args([
                "repoquery",
                "--installed",
                "--unneeded",
                "--qf",
                "%{name}\t%{version}-%{release}",
            ])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return Ok(Vec::new()),
        };

        let mut list = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&name_str) = parts.first() {
                if let Ok(pkg_name) = PackageName::new(name_str) {
                    let category = PackageCategory::classify_dependency(name_str);
                    let mut entry =
                        PackageEntry::new(pkg_name, PackageSource::DnfNative, PackageScope::System)
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

        let cmd = if which::which("dnf5").is_ok() {
            "dnf5"
        } else {
            "dnf"
        };
        let name_strs: Vec<&str> = names.iter().map(|n| n.as_str()).collect();
        let status = Command::new("sudo")
            .arg(cmd)
            .args(["install", "-y"])
            .args(&name_strs)
            .status()
            .map_err(|e| DomainError::PackageError {
                message: format!("Failed to execute dnf install: {}", e),
            })?;

        if !status.success() {
            return Err(DomainError::PackageError {
                message: "DNF batch installation failed".to_string(),
            });
        }

        Ok(names)
    }

    fn is_available(&self, name: &PackageName) -> Result<bool, DomainError> {
        let cmd = if which::which("dnf5").is_ok() {
            "dnf5"
        } else {
            "dnf"
        };
        let status = Command::new(cmd).args(["info", name.as_str()]).output();

        match status {
            Ok(out) => Ok(out.status.success()),
            Err(_) => Ok(false),
        }
    }
}

/// Adapter for APT / DPKG native packages (Debian, Ubuntu, Pop!_OS, Mint)
#[derive(Debug, Default, Clone)]
pub struct AptAdapter;

impl AptAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PackagePort for AptAdapter {
    fn source(&self) -> PackageSource {
        PackageSource::AptNative
    }

    fn query_installed(&self) -> Result<Vec<PackageEntry>, DomainError> {
        if which::which("apt-mark").is_err() && which::which("dpkg-query").is_err() {
            return Ok(Vec::new());
        }

        let manual_pkgs: std::collections::HashSet<String> = if which::which("apt-mark").is_ok() {
            if let Ok(out) = Command::new("apt-mark").arg("showmanual").output() {
                if out.status.success() {
                    String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .collect()
                } else {
                    std::collections::HashSet::new()
                }
            } else {
                std::collections::HashSet::new()
            }
        } else {
            std::collections::HashSet::new()
        };

        let output = if which::which("dpkg-query").is_ok() {
            Command::new("dpkg-query")
                .args(["-W", "-f=${Package}\t${Version}\t${Status}\n"])
                .output()
        } else {
            return Ok(Vec::new());
        };

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return Ok(Vec::new()),
        };

        let mut list = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if let Some(&name_str) = parts.first() {
                let status = parts.get(2).copied().unwrap_or("");
                if !status.contains("installed") && !status.is_empty() {
                    continue;
                }

                if !manual_pkgs.is_empty() && !manual_pkgs.contains(name_str) {
                    continue;
                }

                if let Ok(pkg_name) = PackageName::new(name_str) {
                    let (mut scope, mut category) =
                        PackageCategory::classify(name_str, PackageSource::AptNative);
                    if is_distro_preinstalled_on_system(name_str) && scope == PackageScope::User {
                        scope = PackageScope::System;
                        category = PackageCategory::DistroBase;
                    }
                    let mut entry = PackageEntry::new(pkg_name, PackageSource::AptNative, scope)
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
        if which::which("apt-mark").is_err() {
            return Ok(Vec::new());
        }

        let output = Command::new("apt-mark").arg("showauto").output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return Ok(Vec::new()),
        };

        let mut list = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let name_str = line.trim();
            if name_str.is_empty() {
                continue;
            }
            if let Ok(pkg_name) = PackageName::new(name_str) {
                let category = PackageCategory::classify_dependency(name_str);
                let entry =
                    PackageEntry::new(pkg_name, PackageSource::AptNative, PackageScope::System)
                        .with_category(category)
                        .as_dependency(true);
                list.push(entry);
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
            .env("DEBIAN_FRONTEND", "noninteractive")
            .arg("apt-get")
            .args(["install", "-y", "--no-install-recommends"])
            .args(&name_strs)
            .status()
            .map_err(|e| DomainError::PackageError {
                message: format!("Failed to execute apt-get install: {}", e),
            })?;

        if !status.success() {
            return Err(DomainError::PackageError {
                message: "APT batch installation failed".to_string(),
            });
        }

        Ok(names)
    }

    fn is_available(&self, name: &PackageName) -> Result<bool, DomainError> {
        let status = Command::new("apt-cache")
            .args(["show", name.as_str()])
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
                    let (scope, category) =
                        PackageCategory::classify(app_id, PackageSource::Flatpak);
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
