use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a user-specified custom directory to be included and compressed in the snapshot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomDirectoryEntry {
    pub id: String,
    pub source_path: PathBuf,
    pub archive_prefix: PathBuf,
    pub label: String,
    pub exclude_patterns: Vec<String>,
    pub size_bytes: u64,
    pub file_count: usize,
}

impl CustomDirectoryEntry {
    pub fn new(source_path: PathBuf, archive_prefix: PathBuf, label: impl Into<String>) -> Self {
        Self {
            id: uuid_or_slug(&source_path),
            source_path,
            archive_prefix,
            label: label.into(),
            exclude_patterns: Vec::new(),
            size_bytes: 0,
            file_count: 0,
        }
    }

    pub fn with_stats(mut self, size_bytes: u64, file_count: usize) -> Self {
        self.size_bytes = size_bytes;
        self.file_count = file_count;
        self
    }
}

fn uuid_or_slug(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("custom_dir")
        .to_string()
}
