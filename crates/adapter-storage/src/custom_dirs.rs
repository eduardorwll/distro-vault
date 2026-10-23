use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use domain::entities::custom_directory::CustomDirectoryEntry;
use domain::errors::DomainError;

pub struct CustomDirsHandler;

impl CustomDirsHandler {
    /// Inspects a user directory, computes its disk size and file count, and generates a valid CustomDirectoryEntry
    pub fn inspect_directory(
        source_dir: &Path,
        label_override: Option<&str>,
    ) -> Result<CustomDirectoryEntry, DomainError> {
        if !source_dir.exists() {
            return Err(DomainError::ValidationError {
                field: "source_dir",
                reason: format!("Custom directory does not exist: {:?}", source_dir),
            });
        }

        if !source_dir.is_dir() {
            return Err(DomainError::ValidationError {
                field: "source_dir",
                reason: format!("Custom path is not a directory: {:?}", source_dir),
            });
        }

        let dir_name = source_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("custom_dir");

        let label = label_override.unwrap_or(dir_name).to_string();
        let archive_prefix = PathBuf::from(format!("custom/{}", dir_name));

        let mut total_size = 0u64;
        let mut total_files = 0usize;

        for entry_res in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
            if entry_res.file_type().is_file() {
                if let Ok(meta) = entry_res.metadata() {
                    total_size += meta.len();
                    total_files += 1;
                }
            }
        }

        let entry = CustomDirectoryEntry::new(source_dir.to_path_buf(), archive_prefix, label)
            .with_stats(total_size, total_files);

        Ok(entry)
    }

    /// Converts CustomDirectoryEntry into source-to-archive path pairs
    pub fn to_archive_item(entry: &CustomDirectoryEntry) -> (PathBuf, PathBuf) {
        (entry.source_path.clone(), entry.archive_prefix.clone())
    }
}
