use crate::entities::ssh_key::SshKeyEntry;
use crate::errors::DomainError;
use std::path::Path;

/// Hexagonal port interface for SSH keys scanning, inspection and permission-hardened restoration
pub trait SshPort: Send + Sync {
    /// Scans the target ~/.ssh directory and returns all discovered keys and configs
    fn scan_ssh_keys(&self, home: &Path) -> Result<Vec<SshKeyEntry>, DomainError>;

    /// Restores SSH keys into ~/.ssh, enforcing mode 0700 on ~/.ssh
    /// and strict POSIX permissions (0600 for private keys/config, 0644 for public keys)
    fn restore_ssh_keys(
        &self,
        staging_ssh_dir: &Path,
        target_home: &Path,
        keys: &[SshKeyEntry],
        dry_run: bool,
    ) -> Result<usize, DomainError>;
}
