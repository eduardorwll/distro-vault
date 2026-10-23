use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::value_objects::{Checksum, UnixMode};

/// Metadata and state of an AppImage portable binary
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppImageEntry {
    pub name: String,
    pub relative_path: PathBuf,
    pub size_bytes: u64,
    pub mode: UnixMode,
    pub checksum: Option<Checksum>,
    pub desktop_file: Option<PathBuf>,
    pub icon_file: Option<PathBuf>,
    pub requires_fuse2: bool,
}

impl AppImageEntry {
    pub fn new(name: impl Into<String>, relative_path: PathBuf, size_bytes: u64, mode: UnixMode) -> Self {
        Self {
            name: name.into(),
            relative_path,
            size_bytes,
            mode,
            checksum: None,
            desktop_file: None,
            icon_file: None,
            requires_fuse2: true, // Most Type-2 AppImages require libfuse2
        }
    }
}
