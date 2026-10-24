use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::value_objects::ProfileId;

/// Represents a discrete Zen Browser Space (Workspace)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub container_id: Option<String>,
    pub open_tabs_count: usize,
    pub pinned_tabs_count: usize,
}

/// Represents an isolated Container tab definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
}

/// Session telemetry and integrity details extracted from mozLz4
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub uncompressed_size: usize,
    pub compressed_size: usize,
    pub windows_count: usize,
    pub spaces_count: usize,
    pub total_tabs: usize,
}

/// Complete representation of an analyzed Zen Browser Profile
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZenProfile {
    pub id: ProfileId,
    pub is_default: bool,
    pub root_path: PathBuf,
    pub session_info: Option<SessionInfo>,
    pub spaces: Vec<Space>,
    pub containers: Vec<Container>,
    pub extensions_count: usize,
    pub has_passwords: bool,
    pub has_bookmarks: bool,
}

impl ZenProfile {
    pub fn new(id: ProfileId, is_default: bool, root_path: PathBuf) -> Self {
        Self {
            id,
            is_default,
            root_path,
            session_info: None,
            spaces: Vec::new(),
            containers: Vec::new(),
            extensions_count: 0,
            has_passwords: false,
            has_bookmarks: false,
        }
    }

    pub fn total_tabs(&self) -> usize {
        self.spaces
            .iter()
            .map(|s| s.open_tabs_count + s.pinned_tabs_count)
            .sum()
    }
}
