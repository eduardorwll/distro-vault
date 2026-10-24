use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use domain::errors::DomainError;
use domain::ports::StoragePort;
use domain::value_objects::Checksum;

/// DTO summarizing the integrity verification of a snapshot archive
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub snapshot_id: String,
    pub computed_checksum: Checksum,
    pub declared_checksum: Option<Checksum>,
    pub is_valid: bool,
    pub total_files: usize,
}

pub struct VerifyIntegrityUseCase {
    storage_port: Arc<dyn StoragePort>,
}

impl VerifyIntegrityUseCase {
    pub fn new(storage_port: Arc<dyn StoragePort>) -> Self {
        Self { storage_port }
    }

    pub fn execute(&self, archive_path: &Path) -> Result<IntegrityReport, DomainError> {
        let computed_checksum = self.storage_port.compute_checksum(archive_path)?;
        let manifest = self.storage_port.read_manifest(archive_path)?;

        let is_valid = match &manifest.checksum {
            Some(declared) => declared == &computed_checksum,
            None => true, // Valid if no checksum was stored during draft
        };

        if !is_valid {
            return Err(DomainError::ChecksumMismatch {
                path: archive_path.to_string_lossy().to_string(),
                expected: manifest
                    .checksum
                    .as_ref()
                    .map(|c| c.as_str())
                    .unwrap_or("none")
                    .to_string(),
                actual: computed_checksum.as_str().to_string(),
            });
        }

        Ok(IntegrityReport {
            snapshot_id: manifest.snapshot_id,
            computed_checksum,
            declared_checksum: manifest.checksum,
            is_valid,
            total_files: manifest.total_files,
        })
    }
}
