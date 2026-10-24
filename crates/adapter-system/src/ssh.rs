use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use domain::entities::ssh_key::{SshFileType, SshKeyEntry};
use domain::errors::DomainError;
use domain::ports::SshPort;

/// Adapter implementing SSH key detection, classification, and secure permission-hardened restoration
#[derive(Debug, Default, Clone)]
pub struct LinuxSshAdapter;

impl LinuxSshAdapter {
    pub fn new() -> Self {
        Self
    }

    fn classify_file(file_name: &str, file_path: &Path) -> SshFileType {
        if file_name.ends_with("-cert.pub") {
            return SshFileType::Certificate;
        }
        if file_name.ends_with(".pub") {
            return SshFileType::PublicKey;
        }
        if file_name == "config" || file_name.starts_with("config.d") || file_name.starts_with("config_") {
            return SshFileType::Config;
        }
        if file_name == "authorized_keys" || file_name.starts_with("authorized_keys") {
            return SshFileType::AuthorizedKeys;
        }
        if file_name == "known_hosts" || file_name.starts_with("known_hosts") {
            return SshFileType::KnownHosts;
        }

        // Inspect header for private key markers
        if let Ok(mut f) = fs::File::open(file_path) {
            let mut buf = [0u8; 256];
            if let Ok(n) = f.read(&mut buf) {
                let header = String::from_utf8_lossy(&buf[..n]);
                if header.contains("PRIVATE KEY") {
                    return SshFileType::PrivateKey;
                }
            }
        }

        // Standard default private key filenames
        if file_name == "id_ed25519"
            || file_name == "id_rsa"
            || file_name == "id_ecdsa"
            || file_name == "id_dsa"
            || file_name == "identity"
        {
            return SshFileType::PrivateKey;
        }

        SshFileType::Other
    }
}

impl SshPort for LinuxSshAdapter {
    fn scan_ssh_keys(&self, home: &Path) -> Result<Vec<SshKeyEntry>, DomainError> {
        let ssh_dir = home.join(".ssh");
        if !ssh_dir.exists() || !ssh_dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let read_dir = fs::read_dir(&ssh_dir).map_err(|e| DomainError::SshError {
            message: format!("Failed to read SSH directory '{}': {}", ssh_dir.display(), e),
        })?;

        for entry_res in read_dir {
            let entry = entry_res.map_err(|e| DomainError::SshError {
                message: format!("Failed reading entry in '{}': {}", ssh_dir.display(), e),
            })?;

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = match entry.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };

            // Ignore temporary editor files or socket pipes
            if file_name.ends_with('~') || file_name.ends_with(".tmp") || file_name.starts_with('.') {
                continue;
            }

            let metadata = fs::metadata(&path).map_err(|e| DomainError::SshError {
                message: format!("Failed to read metadata for '{}': {}", path.display(), e),
            })?;

            let size_bytes = metadata.len();
            let posix_mode = metadata.permissions().mode() & 0o777;
            let file_type = Self::classify_file(&file_name, &path);

            entries.push(SshKeyEntry::new(
                file_name.clone(),
                file_name,
                file_type,
                size_bytes,
                posix_mode,
            ));
        }

        entries.sort_by(|a, b| a.file_name.cmp(&b.file_name));
        Ok(entries)
    }

    fn restore_ssh_keys(
        &self,
        staging_ssh_dir: &Path,
        target_home: &Path,
        keys: &[SshKeyEntry],
        dry_run: bool,
    ) -> Result<usize, DomainError> {
        if keys.is_empty() || !staging_ssh_dir.exists() {
            return Ok(0);
        }

        let target_ssh = target_home.join(".ssh");

        if dry_run {
            let count = keys
                .iter()
                .filter(|k| staging_ssh_dir.join(&k.relative_path).exists())
                .count();
            return Ok(count);
        }

        // 1. Create ~/.ssh directory with strict 0700 permissions
        if !target_ssh.exists() {
            fs::create_dir_all(&target_ssh).map_err(|e| DomainError::SshError {
                message: format!("Failed to create SSH directory '{}': {}", target_ssh.display(), e),
            })?;
        }
        let _ = fs::set_permissions(&target_ssh, fs::Permissions::from_mode(0o700));

        let mut restored_count = 0;

        for key in keys {
            let source_file = staging_ssh_dir.join(&key.relative_path);
            let target_file = target_ssh.join(&key.relative_path);

            if !source_file.exists() {
                continue;
            }

            // Ensure parent directory exists inside .ssh
            if let Some(parent) = target_file.parent() {
                if !parent.exists() {
                    let _ = fs::create_dir_all(parent);
                    let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
                }
            }

            // Copy file content
            fs::copy(&source_file, &target_file).map_err(|e| DomainError::SshError {
                message: format!("Failed copying SSH key to '{}': {}", target_file.display(), e),
            })?;

            // Enforce strict POSIX permissions
            let target_mode = match key.file_type {
                SshFileType::PrivateKey | SshFileType::Config | SshFileType::AuthorizedKeys => 0o600,
                SshFileType::PublicKey | SshFileType::KnownHosts | SshFileType::Certificate => 0o644,
                SshFileType::Other => 0o600,
            };

            let _ = fs::set_permissions(&target_file, fs::Permissions::from_mode(target_mode));
            restored_count += 1;
        }

        Ok(restored_count)
    }
}
