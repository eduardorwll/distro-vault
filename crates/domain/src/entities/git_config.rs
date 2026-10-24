use serde::{Deserialize, Serialize};

/// Metadata and summary for a discovered global Git configuration file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitConfigEntry {
    /// Relative path from user home (e.g. ".gitconfig" or ".config/git/config")
    pub relative_path: String,
    /// Friendly filename (e.g. ".gitconfig" or "config")
    pub file_name: String,
    /// Discovered Git author name (if present)
    pub user_name: Option<String>,
    /// Discovered Git author email (if present)
    pub user_email: Option<String>,
    /// File size in bytes
    pub size_bytes: u64,
}

impl GitConfigEntry {
    pub fn new(
        relative_path: impl Into<String>,
        file_name: impl Into<String>,
        user_name: Option<String>,
        user_email: Option<String>,
        size_bytes: u64,
    ) -> Self {
        Self {
            relative_path: relative_path.into(),
            file_name: file_name.into(),
            user_name,
            user_email,
            size_bytes,
        }
    }
}
