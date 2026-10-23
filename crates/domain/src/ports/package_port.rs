use crate::entities::package::PackageSource;
use crate::entities::PackageEntry;
use crate::errors::DomainError;
use crate::value_objects::PackageName;

/// Hexagonal port interface for package management backends (Pacman, AUR, Flatpak, DNF, APT, Nix)
pub trait PackagePort: Send + Sync {
    /// Identifies the package source handled by this port implementation
    fn source(&self) -> PackageSource;

    /// Queries explicitly installed packages for this backend
    fn query_installed(&self) -> Result<Vec<PackageEntry>, DomainError>;

    /// Installs a batch of packages
    fn install_batch(
        &self,
        packages: &[PackageEntry],
        dry_run: bool,
    ) -> Result<Vec<PackageName>, DomainError>;

    /// Validates if a given package name exists in the repositories
    fn is_available(&self, name: &PackageName) -> Result<bool, DomainError>;
}
