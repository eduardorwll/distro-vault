use std::path::Path;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use domain::errors::DomainError;
use domain::ports::StoragePort;
use domain::value_objects::{Checksum, DistroFamily};

/// DTO representing the high-level summary of an inspected snapshot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotSummary {
    pub snapshot_id: String,
    pub created_at_utc: String,
    pub source_host: String,
    pub source_distro: DistroFamily,
    pub zen_profiles_count: usize,
    pub total_tabs: usize,
    pub spaces_count: usize,
    pub packages_count: usize,
    pub skills_count: usize,
    pub custom_dirs_count: usize,
    pub checksum: Option<Checksum>,
}

pub struct InspectSnapshotUseCase {
    storage_port: Arc<dyn StoragePort>,
}

impl InspectSnapshotUseCase {
    pub fn new(storage_port: Arc<dyn StoragePort>) -> Self {
        Self { storage_port }
    }

    pub fn execute(&self, archive_path: &Path) -> Result<SnapshotSummary, DomainError> {
        let manifest = self.storage_port.read_manifest(archive_path)?;

        let total_tabs: usize = manifest.zen_profiles.iter().map(|p| p.total_tabs()).sum();
        let spaces_count: usize = manifest.zen_profiles.iter().map(|p| p.spaces.len()).sum();

        Ok(SnapshotSummary {
            snapshot_id: manifest.snapshot_id,
            created_at_utc: manifest.created_at_utc,
            source_host: manifest.source_host,
            source_distro: manifest.source_distro,
            zen_profiles_count: manifest.zen_profiles.len(),
            total_tabs,
            spaces_count,
            packages_count: manifest.packages.len(),
            skills_count: manifest.agent_governance.skills.len(),
            custom_dirs_count: manifest.custom_directories.len(),
            checksum: manifest.checksum,
        })
    }
}
