use std::path::Path;

use crate::entities::VaultConfig;
use crate::errors::DomainError;

/// Hexagonal port interface for loading, persisting and querying user configuration
pub trait ConfigPort: Send + Sync {
    /// Loads user configuration from storage, initializing with sensible defaults if missing
    fn load_config(&self) -> Result<VaultConfig, DomainError>;

    /// Persists user configuration changes to disk
    fn save_config(&self, config: &VaultConfig) -> Result<(), DomainError>;

    /// Returns the canonical path to the configuration file on the system
    fn config_path(&self) -> &Path;
}
