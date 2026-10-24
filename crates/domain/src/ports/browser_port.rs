use std::path::Path;

use crate::entities::ZenProfile;
use crate::errors::DomainError;
use crate::value_objects::ProfileId;

/// Hexagonal port interface for browser operations (Zen Browser / Firefox architecture)
pub trait BrowserPort: Send + Sync {
    /// Checks whether the browser is running or has an active .parentlock
    fn is_locked(&self, profile_root: &Path) -> Result<bool, DomainError>;

    /// Scans the config root (e.g. ~/.config/zen) and returns detected profiles
    fn scan_profiles(&self, config_root: &Path) -> Result<Vec<ZenProfile>, DomainError>;

    /// Decompresses a mozlz4 binary payload (zen-sessions.jsonlz4) into a JSON string
    fn decompress_session(&self, compressed_bytes: &[u8]) -> Result<String, DomainError>;

    /// Compresses a JSON string into a valid mozlz4 binary payload with magic header
    fn compress_session(&self, json_payload: &str) -> Result<Vec<u8>, DomainError>;

    /// Normalizes installs.ini and profiles.ini to guarantee the restored profile is set as Default=1
    fn normalize_installation_hash(
        &self,
        config_root: &Path,
        target_profile_id: &ProfileId,
    ) -> Result<(), DomainError>;

    /// Exports essential browser state to destination directory
    fn export_profile_state(
        &self,
        profile: &ZenProfile,
        dest_dir: &Path,
    ) -> Result<(), DomainError>;

    /// Restores browser state from source directory into target config root
    fn restore_profile_state(
        &self,
        source_dir: &Path,
        config_root: &Path,
    ) -> Result<(), DomainError>;
}
