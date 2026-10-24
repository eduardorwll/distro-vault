use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use domain::entities::git_config::GitConfigEntry;
use domain::errors::DomainError;
use domain::ports::GitConfigPort;

/// Adapter implementing global Git configuration discovery, metadata extraction, and safe restoration
#[derive(Debug, Default, Clone)]
pub struct LinuxGitConfigAdapter;

impl LinuxGitConfigAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Extracts `user.name` and `user.email` from an INI-style Git config file
    fn extract_user_identity(file_path: &Path) -> (Option<String>, Option<String>) {
        let file = match fs::File::open(file_path) {
            Ok(f) => f,
            Err(_) => return (None, None),
        };

        let reader = BufReader::new(file);
        let mut in_user_section = false;
        let mut name = None;
        let mut email = None;

        for line_res in reader.lines() {
            let line = match line_res {
                Ok(l) => l.trim().to_string(),
                Err(_) => continue,
            };

            // Ignore comments and empty lines
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // Section header
            if line.starts_with('[') && line.ends_with(']') {
                let section_name = line[1..line.len() - 1].trim().to_lowercase();
                in_user_section = section_name == "user";
                continue;
            }

            if in_user_section {
                if let Some((k, v)) = line.split_once('=') {
                    let key = k.trim().to_lowercase();
                    let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
                    if key == "name" && name.is_none() && !val.is_empty() {
                        name = Some(val);
                    } else if key == "email" && email.is_none() && !val.is_empty() {
                        email = Some(val);
                    }
                }
            }
        }

        (name, email)
    }
}

impl GitConfigPort for LinuxGitConfigAdapter {
    fn scan_git_config(&self, home: &Path) -> Result<Vec<GitConfigEntry>, DomainError> {
        let mut entries = Vec::new();

        let candidates = [
            (".gitconfig", ".gitconfig"),
            (".config/git/config", "config"),
        ];

        for (rel_path, file_name) in candidates {
            let full_path = home.join(rel_path);
            if full_path.is_file() {
                let size_bytes = fs::metadata(&full_path)
                    .map(|m| m.len())
                    .unwrap_or(0);

                let (user_name, user_email) = Self::extract_user_identity(&full_path);

                entries.push(GitConfigEntry::new(
                    rel_path,
                    file_name,
                    user_name,
                    user_email,
                    size_bytes,
                ));
            }
        }

        Ok(entries)
    }

    fn restore_git_config(
        &self,
        staging_git_dir: &Path,
        target_home: &Path,
        configs: &[GitConfigEntry],
        dry_run: bool,
    ) -> Result<usize, DomainError> {
        let mut restored_count = 0;

        for cfg in configs {
            let src = staging_git_dir.join(&cfg.relative_path);
            let dest = target_home.join(&cfg.relative_path);

            if !src.exists() {
                continue;
            }

            if !dry_run {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).map_err(|e| {
                        DomainError::GitConfigError {
                            message: format!(
                                "Falha ao criar diretório pai para {}: {}",
                                dest.display(),
                                e
                            ),
                        }
                    })?;
                }

                // Defensive pre-removal to avoid symlink or permission collision
                if dest.exists() {
                    let _ = fs::remove_file(&dest);
                }

                fs::copy(&src, &dest).map_err(|e| {
                    DomainError::GitConfigError {
                        message: format!(
                            "Falha ao restaurar configuração Git em {}: {}",
                            dest.display(),
                            e
                        ),
                    }
                })?;

                // Enforce POSIX 0644 mode
                let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o644));
            }

            restored_count += 1;
        }

        Ok(restored_count)
    }
}
