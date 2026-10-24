use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("Manifest error: {message}")]
    ManifestError { message: String },

    #[error("Checksum mismatch for '{path}': expected '{expected}', found '{actual}'")]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("Browser profile error: {message}")]
    BrowserError { message: String },

    #[error("Browser is locked by process. Close Zen Browser before continuing ({lock_file})")]
    BrowserLocked { lock_file: PathBuf },

    #[error("LZ4 decompression threshold exceeded ({size} bytes exceeds max allowed limit)")]
    DecompressionBombDetected { size: usize },

    #[error("Package manager error: {message}")]
    PackageError { message: String },

    #[error("AppImage validation error: {message}")]
    AppImageError { message: String },

    #[error("Storage security violation: Zip-slip / path traversal detected for '{path}'")]
    SecurityViolation { path: String },

    #[error("SSH operation error: {message}")]
    SshError { message: String },

    #[error("Agent governance error: {message}")]
    AgentGovernanceError { message: String },

    #[error("Unsupported distribution family: '{distro}'")]
    UnsupportedDistro { distro: String },

    #[error("Entity validation error: field '{field}' is invalid: {reason}")]
    ValidationError { field: &'static str, reason: String },
}
