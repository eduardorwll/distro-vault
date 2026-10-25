use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use domain::entities::VaultConfig;
use domain::errors::DomainError;
use domain::ports::ConfigPort;

/// Concrete implementation of ConfigPort persisting configuration in TOML format
#[derive(Debug, Clone)]
pub struct TomlConfigAdapter {
    config_path: PathBuf,
    user_home: PathBuf,
}

impl TomlConfigAdapter {
    /// Creates adapter targeting an explicit path and associated user home directory
    pub fn new(config_path: impl Into<PathBuf>, user_home: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
            user_home: user_home.into(),
        }
    }

    /// Creates adapter targeting the canonical path ~/.config/distro-vault/config.toml
    pub fn default_for_home(home: &Path) -> Self {
        let config_path = home.join(".config/distro-vault/config.toml");
        Self {
            config_path,
            user_home: home.to_path_buf(),
        }
    }
}

impl ConfigPort for TomlConfigAdapter {
    fn load_config(&self) -> Result<VaultConfig, DomainError> {
        if !self.config_path.exists() {
            let default_cfg = VaultConfig::default_for_home(&self.user_home);
            // Attempt to write the initial default config defensively
            let _ = self.save_config(&default_cfg);
            return Ok(default_cfg);
        }

        let content = fs::read_to_string(&self.config_path).map_err(|e| {
            DomainError::ManifestError {
                message: format!(
                    "Failed to read configuration file {:?}: {}",
                    self.config_path, e
                ),
            }
        })?;

        match toml::from_str::<VaultConfig>(&content) {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                // In case of syntax corruption, return default configuration safely
                eprintln!(
                    "Aviso: Erro ao interpretar {:?} ({}). Utilizando configurações padrão.",
                    self.config_path, e
                );
                Ok(VaultConfig::default_for_home(&self.user_home))
            }
        }
    }

    fn save_config(&self, config: &VaultConfig) -> Result<(), DomainError> {
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| DomainError::ManifestError {
                    message: format!(
                        "Failed to create config parent directory {:?}: {}",
                        parent, e
                    ),
                })?;
            }
        }

        let toml_string = toml::to_string_pretty(config).map_err(|e| {
            DomainError::ManifestError {
                message: format!("Failed to serialize VaultConfig to TOML: {}", e),
            }
        })?;

        let mut file = File::create(&self.config_path).map_err(|e| {
            DomainError::ManifestError {
                message: format!(
                    "Failed to create configuration file {:?}: {}",
                    self.config_path, e
                ),
            }
        })?;

        file.write_all(toml_string.as_bytes())
            .map_err(|e| DomainError::ManifestError {
                message: format!(
                    "Failed to write configuration file {:?}: {}",
                    self.config_path, e
                ),
            })?;

        // Set safe POSIX 0644 permissions
        let mut perms = file
            .metadata()
            .map_err(|e| DomainError::ManifestError {
                message: format!("Failed to query config metadata: {}", e),
            })?
            .permissions();
        perms.set_mode(0o644);
        let _ = file.set_permissions(perms);

        Ok(())
    }

    fn config_path(&self) -> &Path {
        &self.config_path
    }
}
