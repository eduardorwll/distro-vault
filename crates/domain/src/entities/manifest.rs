use serde::{Deserialize, Serialize};

use crate::entities::agent_governance::AgentGovernance;
use crate::entities::appimage::AppImageEntry;
use crate::entities::custom_directory::CustomDirectoryEntry;
use crate::entities::package::PackageEntry;
use crate::entities::zen_profile::ZenProfile;
use crate::value_objects::{Checksum, DistroFamily};

/// Complete schema and manifest of a Distro-Vault snapshot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub snapshot_id: String,
    pub created_at_utc: String,
    pub source_host: String,
    pub source_distro: DistroFamily,
    pub zen_profiles: Vec<ZenProfile>,
    pub packages: Vec<PackageEntry>,
    pub appimages: Vec<AppImageEntry>,
    pub custom_directories: Vec<CustomDirectoryEntry>,
    pub agent_governance: AgentGovernance,
    pub checksum: Option<Checksum>,
    pub total_files: usize,
    pub total_size_bytes: u64,
}

impl Manifest {
    pub fn new(
        snapshot_id: impl Into<String>,
        created_at_utc: impl Into<String>,
        source_host: impl Into<String>,
        source_distro: DistroFamily,
        agent_governance: AgentGovernance,
    ) -> Self {
        Self {
            version: "1.0.0".to_string(),
            snapshot_id: snapshot_id.into(),
            created_at_utc: created_at_utc.into(),
            source_host: source_host.into(),
            source_distro,
            zen_profiles: Vec::new(),
            packages: Vec::new(),
            appimages: Vec::new(),
            custom_directories: Vec::new(),
            agent_governance,
            checksum: None,
            total_files: 0,
            total_size_bytes: 0,
        }
    }
}
