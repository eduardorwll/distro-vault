use std::path::{Path, PathBuf};

use crate::entities::Manifest;
use crate::errors::DomainError;
use crate::value_objects::Checksum;

/// Hexagonal port interface for snapshot archiving, compression and extraction
pub trait StoragePort: Send + Sync {
    /// Creates an atomic snapshot archive (.tar.zst) preserving POSIX permissions
    fn create_snapshot(
        &self,
        source_items: &[(PathBuf, PathBuf)], // (source_path, path_in_archive)
        manifest: &Manifest,
        output_archive: &Path,
    ) -> Result<Checksum, DomainError>;

    /// Reads and deserializes the Manifest from an existing snapshot without extracting everything
    fn read_manifest(&self, archive: &Path) -> Result<Manifest, DomainError>;

    /// Extracts snapshot contents into destination directory with Zip-Slip protection
    fn extract_snapshot(&self, archive: &Path, target_dir: &Path) -> Result<(), DomainError>;

    /// Computes the SHA-256 checksum of any file
    fn compute_checksum(&self, path: &Path) -> Result<Checksum, DomainError>;
}
