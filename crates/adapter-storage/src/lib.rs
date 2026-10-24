pub mod archive;
pub mod custom_dirs;

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use domain::entities::Manifest;
use domain::errors::DomainError;
use domain::ports::StoragePort;
use domain::value_objects::Checksum;

pub use archive::{TarZstdPacker, ZipSlipGuard};
pub use custom_dirs::CustomDirsHandler;

#[derive(Debug, Default, Clone)]
pub struct ZstdStorageAdapter;

impl ZstdStorageAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl StoragePort for ZstdStorageAdapter {
    fn create_snapshot(
        &self,
        source_items: &[(PathBuf, PathBuf)],
        manifest: &Manifest,
        output_archive: &Path,
    ) -> Result<Checksum, DomainError> {
        let manifest_json =
            serde_json::to_string_pretty(manifest).map_err(|e| DomainError::ManifestError {
                message: format!("Failed to serialize manifest: {}", e),
            })?;

        TarZstdPacker::pack(source_items, &manifest_json, output_archive)?;
        self.compute_checksum(output_archive)
    }

    fn read_manifest(&self, archive: &Path) -> Result<Manifest, DomainError> {
        let json_content = TarZstdPacker::read_manifest_json(archive)?;
        serde_json::from_str(&json_content).map_err(|e| DomainError::ManifestError {
            message: format!("Failed to deserialize manifest.json from archive: {}", e),
        })
    }

    fn extract_snapshot(&self, archive: &Path, target_dir: &Path) -> Result<(), DomainError> {
        TarZstdPacker::unpack(archive, target_dir)
    }

    fn compute_checksum(&self, path: &Path) -> Result<Checksum, DomainError> {
        let file = File::open(path).map_err(|e| DomainError::ManifestError {
            message: format!(
                "Failed to open file for checksum calculation {:?}: {}",
                path, e
            ),
        })?;

        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .map_err(|e| DomainError::ManifestError {
                    message: format!(
                        "Failed to read stream for checksum calculation {:?}: {}",
                        path, e
                    ),
                })?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        let hex_str = format!("{:x}", result);
        Checksum::new(hex_str)
    }
}
