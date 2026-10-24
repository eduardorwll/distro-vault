use std::path::Path;

use crate::entities::git_config::GitConfigEntry;
use crate::errors::DomainError;

/// Port interface for scanning and restoring global Git configuration
pub trait GitConfigPort: Send + Sync {
    /// Scans the target home directory for global Git configuration files (.gitconfig, .config/git/config)
    fn scan_git_config(&self, home: &Path) -> Result<Vec<GitConfigEntry>, DomainError>;

    /// Restores Git configuration files from staging into target home directory
    fn restore_git_config(
        &self,
        staging_git_dir: &Path,
        target_home: &Path,
        configs: &[GitConfigEntry],
        dry_run: bool,
    ) -> Result<usize, DomainError>;
}
