use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Declarative user configuration managing I/O directories and TUI ergonomics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Directories scanned for snapshot archives (.tar.zst)
    pub input_dirs: Vec<PathBuf>,
    /// Target directory where newly created snapshots and sub-dumps are written
    pub output_dir: PathBuf,
    /// Whether packages should be automatically installed by default during restoration
    pub auto_install_packages: bool,
    /// Whether the TUI tree renders in compact (minimalist/de-bloated) or detailed mode
    pub compact_view: bool,
    /// Optional cache directory for staging extracted archives
    pub staging_root: Option<PathBuf>,
}

impl Default for VaultConfig {
    fn default() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/home/user"));

        Self::default_for_home(&home)
    }
}

impl VaultConfig {
    /// Instantiates default configuration anchored to a given user home directory
    pub fn default_for_home(home: &Path) -> Self {
        let default_backup = home.join("Antigravity/backups");
        let local_backup = PathBuf::from("./backups");

        Self {
            input_dirs: vec![default_backup.clone(), local_backup],
            output_dir: default_backup,
            auto_install_packages: true,
            compact_view: false,
            staging_root: None,
        }
    }

    /// Creates a new custom VaultConfig with an explicit output directory
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        let out = output_dir.into();
        Self {
            input_dirs: vec![out.clone()],
            output_dir: out,
            auto_install_packages: true,
            compact_view: false,
            staging_root: None,
        }
    }

    /// Adds a directory to the scanned input directories (ignores duplicates)
    pub fn add_input_dir(&mut self, dir: impl Into<PathBuf>) -> bool {
        let path = dir.into();
        if !self.input_dirs.contains(&path) {
            self.input_dirs.push(path);
            true
        } else {
            false
        }
    }

    /// Removes an input directory from the search list
    pub fn remove_input_dir(&mut self, dir: &Path) -> bool {
        let initial_len = self.input_dirs.len();
        self.input_dirs.retain(|d| d != dir);
        self.input_dirs.len() < initial_len
    }

    /// Sets the default output directory for new snapshots and slices
    pub fn set_output_dir(&mut self, dir: impl Into<PathBuf>) {
        let path = dir.into();
        self.add_input_dir(path.clone());
        self.output_dir = path;
    }

    /// Toggles the compact view density flag
    pub fn toggle_compact_view(&mut self) {
        self.compact_view = !self.compact_view;
    }

    /// Toggles the automatic package installation flag
    pub fn toggle_auto_install_packages(&mut self) {
        self.auto_install_packages = !self.auto_install_packages;
    }
}
