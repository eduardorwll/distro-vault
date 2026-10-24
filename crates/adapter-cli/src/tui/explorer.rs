use ratatui::widgets::ListState;
use std::fs;
use std::path::{Path, PathBuf};

/// Purpose of the file explorer modal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExplorerTarget {
    /// Select a directory to include in the backup snapshot
    #[default]
    CustomDir,
    /// Select a .tar.zst snapshot archive OR a folder containing backups to scan
    SnapshotOrFolder,
}

/// The kind of entry listed in the explorer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplorerEntryKind {
    ParentDir,
    Directory,
    SnapshotArchive,
}

/// A single item listed in the explorer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerEntry {
    pub path: PathBuf,
    pub name: String,
    pub kind: ExplorerEntryKind,
    pub size_bytes: u64,
}

/// Outcome when a user confirms an item in the explorer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorerSelection {
    Directory(PathBuf),
    SnapshotFile(PathBuf),
}

/// State for the unified interactive filesystem browser modal
#[derive(Debug, Clone)]
pub struct FileExplorerState {
    pub is_open: bool,
    pub target: ExplorerTarget,
    pub current_dir: PathBuf,
    pub entries: Vec<ExplorerEntry>,
    pub cursor: usize,
    pub list_state: ListState,
}

impl Default for FileExplorerState {
    fn default() -> Self {
        Self::new()
    }
}

impl FileExplorerState {
    pub fn new() -> Self {
        let default_home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/home/eduardorwll"));

        let mut explorer = Self {
            is_open: false,
            target: ExplorerTarget::CustomDir,
            current_dir: default_home,
            entries: Vec::new(),
            cursor: 0,
            list_state: ListState::default(),
        };
        explorer.refresh_entries();
        explorer
    }

    /// Opens the file explorer at a given starting directory for a specific target
    pub fn open(&mut self, start_dir: &Path, target: ExplorerTarget) {
        self.target = target;
        let dir = if start_dir.is_dir() {
            start_dir.to_path_buf()
        } else if let Some(parent) = start_dir.parent() {
            if parent.is_dir() {
                parent.to_path_buf()
            } else {
                std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
            }
        } else {
            std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
        };

        self.current_dir = dir;
        self.cursor = 0;
        self.is_open = true;
        self.refresh_entries();
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Scans `current_dir` and populates `entries` cleanly
    pub fn refresh_entries(&mut self) {
        let mut dirs = Vec::new();
        let mut archives = Vec::new();

        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden metadata directories/files to keep the view clean
                if file_name.starts_with('.') {
                    continue;
                }

                if path.is_dir() {
                    dirs.push(ExplorerEntry {
                        path: path.clone(),
                        name: file_name,
                        kind: ExplorerEntryKind::Directory,
                        size_bytes: 0,
                    });
                } else if self.target == ExplorerTarget::SnapshotOrFolder
                    && file_name.ends_with(".tar.zst")
                {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    archives.push(ExplorerEntry {
                        path: path.clone(),
                        name: file_name,
                        kind: ExplorerEntryKind::SnapshotArchive,
                        size_bytes: size,
                    });
                }
            }
        }

        dirs.sort_by_key(|a| a.name.to_lowercase());
        archives.sort_by_key(|a| a.name.to_lowercase());

        let mut all = Vec::new();
        // Add parent directory ".." if not at filesystem root
        if let Some(parent) = self.current_dir.parent() {
            all.push(ExplorerEntry {
                path: parent.to_path_buf(),
                name: ".. (Diretório anterior)".to_string(),
                kind: ExplorerEntryKind::ParentDir,
                size_bytes: 0,
            });
        }

        all.extend(dirs);
        all.extend(archives);

        self.entries = all;
        if self.cursor >= self.entries.len() {
            self.cursor = self.entries.len().saturating_sub(1);
        }
        self.list_state.select(if self.entries.is_empty() {
            None
        } else {
            Some(self.cursor)
        });
    }

    pub fn next(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        if self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
        } else {
            self.cursor = 0;
        }
        self.list_state.select(Some(self.cursor));
    }

    pub fn prev(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        } else {
            self.cursor = self.entries.len() - 1;
        }
        self.list_state.select(Some(self.cursor));
    }

    /// Moves up to the parent directory
    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.cursor = 0;
            self.refresh_entries();
        }
    }

    /// Action triggered on [Enter]: opens directory or selects snapshot file
    pub fn enter_or_select(&mut self) -> Option<ExplorerSelection> {
        let entry = self.entries.get(self.cursor)?.clone();
        match entry.kind {
            ExplorerEntryKind::ParentDir => {
                self.current_dir = entry.path;
                self.cursor = 0;
                self.refresh_entries();
                None
            }
            ExplorerEntryKind::Directory => {
                self.current_dir = entry.path;
                self.cursor = 0;
                self.refresh_entries();
                None
            }
            ExplorerEntryKind::SnapshotArchive => {
                self.is_open = false;
                Some(ExplorerSelection::SnapshotFile(entry.path))
            }
        }
    }

    /// Action triggered on [Space] or [S]: confirms selection of the highlighted directory or current directory
    pub fn confirm_selection(&mut self) -> Option<ExplorerSelection> {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.kind == ExplorerEntryKind::Directory {
                self.is_open = false;
                return Some(ExplorerSelection::Directory(entry.path.clone()));
            }
            if entry.kind == ExplorerEntryKind::SnapshotArchive {
                self.is_open = false;
                return Some(ExplorerSelection::SnapshotFile(entry.path.clone()));
            }
        }

        // If cursor is on ".." or empty, select current_dir
        self.is_open = false;
        Some(ExplorerSelection::Directory(self.current_dir.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_explorer_navigation_and_selection() {
        let tmp = tempdir().expect("Failed to create tempdir");
        let root = tmp.path();

        let sub_a = root.join("alpha");
        let sub_b = root.join("beta");
        fs::create_dir_all(&sub_a).unwrap();
        fs::create_dir_all(&sub_b).unwrap();

        let snap_file = root.join("distro-vault-123.tar.zst");
        fs::write(&snap_file, b"MOCK ARCHIVE").unwrap();

        let mut explorer = FileExplorerState::new();

        // 1. Test opening in CustomDir mode (ignores .tar.zst, shows directories)
        explorer.open(root, ExplorerTarget::CustomDir);
        assert!(explorer.is_open);
        assert_eq!(explorer.current_dir, root);

        let names: Vec<_> = explorer.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
        assert!(!names.contains(&"distro-vault-123.tar.zst"));

        // 2. Test opening in SnapshotOrFolder mode (shows both directories and .tar.zst)
        explorer.open(root, ExplorerTarget::SnapshotOrFolder);
        let names_snap: Vec<_> = explorer.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names_snap.contains(&"alpha"));
        assert!(names_snap.contains(&"distro-vault-123.tar.zst"));

        // 3. Test enter_or_select into directory
        let alpha_idx = explorer.entries.iter().position(|e| e.name == "alpha").unwrap();
        explorer.cursor = alpha_idx;
        let res = explorer.enter_or_select();
        assert_eq!(res, None);
        assert_eq!(explorer.current_dir, sub_a);

        // 4. Test go_up
        explorer.go_up();
        assert_eq!(explorer.current_dir, root);

        // 5. Test confirming directory selection
        let beta_idx = explorer.entries.iter().position(|e| e.name == "beta").unwrap();
        explorer.cursor = beta_idx;
        let confirmed = explorer.confirm_selection();
        assert_eq!(confirmed, Some(ExplorerSelection::Directory(sub_b)));
        assert!(!explorer.is_open);
    }
}
