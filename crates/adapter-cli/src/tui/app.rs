use ratatui::widgets::ListState;
use std::path::PathBuf;

use adapter_storage::CustomDirsHandler;
use domain::entities::custom_directory::CustomDirectoryEntry;
use domain::entities::manifest::Manifest;
use domain::entities::selection_filter::SelectionFilter;
use domain::value_objects::PackageName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TuiMode {
    #[default]
    Backup,
    Restore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputTarget {
    #[default]
    CustomDir,
    SnapshotPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotItem {
    pub path: PathBuf,
    pub filename: String,
    pub size_bytes: u64,
    pub modified_at: String,
    pub manifest: Option<Manifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiItem {
    pub id: String,
    pub name: String,
    pub details: String,
    pub included: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiRow {
    Header {
        category_id: String,
        title: String,
        count: usize,
    },
    Item {
        category_id: String,
        item: TuiItem,
    },
}

impl TuiRow {
    pub fn is_header(&self) -> bool {
        matches!(self, Self::Header { .. })
    }

    pub fn item(&self) -> Option<&TuiItem> {
        match self {
            Self::Item { item, .. } => Some(item),
            _ => None,
        }
    }

    pub fn item_mut(&mut self) -> Option<&mut TuiItem> {
        match self {
            Self::Item { item, .. } => Some(item),
            _ => None,
        }
    }
}

pub struct TuiApp {
    pub mode: TuiMode,
    pub current_tab: usize,
    pub user_packages: Vec<TuiRow>,
    pub system_packages: Vec<TuiRow>,
    pub dependency_packages: Vec<TuiRow>,
    pub skills: Vec<TuiRow>,
    pub appimages: Vec<TuiRow>,
    pub custom_dirs: Vec<CustomDirectoryEntry>,
    pub zen_spaces: Vec<TuiRow>,
    pub ssh_keys: Vec<TuiRow>,
    pub cursors: [usize; 8],
    pub list_states: [ListState; 8],
    // Restore mode fields
    pub snapshots: Vec<SnapshotItem>,
    pub snapshot_cursor: usize,
    pub snapshot_list_state: ListState,
    pub restore_install_packages: bool,
    pub restore_dry_run: bool,
    pub should_start_restore: bool,
    // Input / Modal fields
    pub input_mode: bool,
    pub input_target: InputTarget,
    pub input_buffer: String,
    pub status_message: Option<String>,
    pub default_backups_dir: PathBuf,
    pub should_quit: bool,
    pub should_start_backup: bool,
}

impl TuiApp {
    pub fn new() -> Self {
        let default_backups_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("backups");

        Self {
            mode: TuiMode::Backup,
            current_tab: 0,
            user_packages: Vec::new(),
            system_packages: Vec::new(),
            dependency_packages: Vec::new(),
            skills: Vec::new(),
            appimages: Vec::new(),
            custom_dirs: Vec::new(),
            zen_spaces: Vec::new(),
            ssh_keys: Vec::new(),
            cursors: [0; 8],
            list_states: [
                ListState::default(),
                ListState::default(),
                ListState::default(),
                ListState::default(),
                ListState::default(),
                ListState::default(),
                ListState::default(),
                ListState::default(),
            ],
            snapshots: Vec::new(),
            snapshot_cursor: 0,
            snapshot_list_state: ListState::default(),
            restore_install_packages: true,
            restore_dry_run: false,
            should_start_restore: false,
            input_mode: false,
            input_target: InputTarget::CustomDir,
            input_buffer: String::new(),
            status_message: None,
            default_backups_dir,
            should_quit: false,
            should_start_backup: false,
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 8;
    }

    pub fn set_mode(&mut self, mode: TuiMode) {
        self.mode = mode;
        if mode == TuiMode::Restore && !self.snapshots.is_empty() {
            self.snapshot_list_state.select(Some(self.snapshot_cursor));
        }
    }

    pub fn toggle_mode(&mut self) {
        match self.mode {
            TuiMode::Backup => self.set_mode(TuiMode::Restore),
            TuiMode::Restore => self.set_mode(TuiMode::Backup),
        }
    }

    pub fn next_snapshot(&mut self) {
        if !self.snapshots.is_empty() {
            self.snapshot_cursor = (self.snapshot_cursor + 1).min(self.snapshots.len() - 1);
            self.snapshot_list_state.select(Some(self.snapshot_cursor));
        }
    }

    pub fn prev_snapshot(&mut self) {
        if self.snapshot_cursor > 0 {
            self.snapshot_cursor -= 1;
            self.snapshot_list_state.select(Some(self.snapshot_cursor));
        }
    }

    pub fn selected_snapshot(&self) -> Option<&SnapshotItem> {
        self.snapshots.get(self.snapshot_cursor)
    }

    pub fn toggle_restore_install_packages(&mut self) {
        self.restore_install_packages = !self.restore_install_packages;
    }

    pub fn toggle_restore_dry_run(&mut self) {
        self.restore_dry_run = !self.restore_dry_run;
    }

    pub fn add_snapshot_file(&mut self, path: PathBuf, manifest: Option<Manifest>) {
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "backup.tar.zst".to_string());
        let size_bytes = path.metadata().map(|m| m.len()).unwrap_or(0);
        let modified_at = manifest
            .as_ref()
            .map(|m| m.created_at_utc.clone())
            .unwrap_or_else(|| "Data desconhecida".to_string());

        self.snapshots.insert(
            0,
            SnapshotItem {
                path,
                filename,
                size_bytes,
                modified_at,
                manifest,
            },
        );
        self.snapshot_cursor = 0;
        self.snapshot_list_state.select(Some(0));
    }

    pub fn prev_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = 7;
        } else {
            self.current_tab -= 1;
        }
    }

    pub fn current_list_len(&self) -> usize {
        match self.current_tab {
            0 => self.user_packages.len(),
            1 => self.system_packages.len(),
            2 => self.dependency_packages.len(),
            3 => self.skills.len(),
            4 => self.appimages.len(),
            5 => self.custom_dirs.len(),
            6 => self.zen_spaces.len(),
            7 => self.ssh_keys.len(),
            _ => 0,
        }
    }

    pub fn count_items(&self, tab_idx: usize) -> usize {
        let count_rows = |rows: &[TuiRow]| rows.iter().filter(|r| !r.is_header()).count();
        match tab_idx {
            0 => count_rows(&self.user_packages),
            1 => count_rows(&self.system_packages),
            2 => count_rows(&self.dependency_packages),
            3 => count_rows(&self.skills),
            4 => count_rows(&self.appimages),
            5 => self.custom_dirs.len(),
            6 => count_rows(&self.zen_spaces),
            7 => count_rows(&self.ssh_keys),
            _ => 0,
        }
    }

    pub fn count_selected_items(&self, tab_idx: usize) -> usize {
        let count_selected = |rows: &[TuiRow]| {
            rows.iter()
                .filter(|r| match r {
                    TuiRow::Item { item, .. } => item.included,
                    _ => false,
                })
                .count()
        };
        match tab_idx {
            0 => count_selected(&self.user_packages),
            1 => count_selected(&self.system_packages),
            2 => count_selected(&self.dependency_packages),
            3 => count_selected(&self.skills),
            4 => count_selected(&self.appimages),
            5 => self.custom_dirs.len(),
            6 => count_selected(&self.zen_spaces),
            7 => count_selected(&self.ssh_keys),
            _ => 0,
        }
    }

    pub fn next_item(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            let cursor = &mut self.cursors[self.current_tab];
            *cursor = (*cursor + 1).min(len - 1);
            self.list_states[self.current_tab].select(Some(*cursor));
        }
    }

    pub fn prev_item(&mut self) {
        let cursor = &mut self.cursors[self.current_tab];
        if *cursor > 0 {
            *cursor -= 1;
            self.list_states[self.current_tab].select(Some(*cursor));
        }
    }

    pub fn toggle_selected(&mut self) {
        let idx = self.cursors[self.current_tab];
        let toggle_in_rows = |rows: &mut Vec<TuiRow>| {
            if let Some(row) = rows.get(idx) {
                match row {
                    TuiRow::Item { .. } => {
                        if let Some(TuiRow::Item { item, .. }) = rows.get_mut(idx) {
                            item.included = !item.included;
                        }
                    }
                    TuiRow::Header { category_id, .. } => {
                        let cat = category_id.clone();
                        let any_included = rows.iter().any(|r| match r {
                            TuiRow::Item {
                                category_id: c,
                                item,
                            } if c == &cat => item.included,
                            _ => false,
                        });
                        let new_state = !any_included;
                        for r in rows.iter_mut() {
                            if let TuiRow::Item {
                                category_id: c,
                                item,
                            } = r
                            {
                                if c == &cat {
                                    item.included = new_state;
                                }
                            }
                        }
                    }
                }
            }
        };

        match self.current_tab {
            0 => toggle_in_rows(&mut self.user_packages),
            1 => toggle_in_rows(&mut self.system_packages),
            2 => toggle_in_rows(&mut self.dependency_packages),
            3 => toggle_in_rows(&mut self.skills),
            4 => toggle_in_rows(&mut self.appimages),
            5 => {
                if idx < self.custom_dirs.len() {
                    self.custom_dirs.remove(idx);
                    if self.cursors[5] > 0 && self.cursors[5] >= self.custom_dirs.len() {
                        self.cursors[5] -= 1;
                    }
                    self.list_states[5].select(Some(self.cursors[5]));
                }
            }
            6 => toggle_in_rows(&mut self.zen_spaces),
            7 => toggle_in_rows(&mut self.ssh_keys),
            _ => {}
        }
    }

    pub fn add_custom_directory(&mut self, path_str: &str) -> Result<(), String> {
        let path = PathBuf::from(path_str.trim());
        let inspected =
            CustomDirsHandler::inspect_directory(&path, None).map_err(|e| format!("{}", e))?;

        self.custom_dirs.push(inspected);
        self.status_message = Some(format!("Diretório adicionado com sucesso: {}", path_str));
        Ok(())
    }

    pub fn build_selection_filter(&self) -> SelectionFilter {
        let mut filter = SelectionFilter::new();

        let scan_packages = |rows: &[TuiRow], filt: &mut SelectionFilter| {
            for row in rows {
                if let TuiRow::Item { item, .. } = row {
                    if !item.included {
                        if let Ok(name) = PackageName::new(&item.id) {
                            filt.exclude_package(name);
                        }
                    }
                }
            }
        };

        scan_packages(&self.user_packages, &mut filter);
        scan_packages(&self.system_packages, &mut filter);

        for row in &self.dependency_packages {
            if let TuiRow::Item { item, .. } = row {
                if item.included {
                    if let Ok(name) = PackageName::new(&item.id) {
                        filter.include_dependency(name);
                    }
                } else if let Ok(name) = PackageName::new(&item.id) {
                    filter.exclude_package(name);
                }
            }
        }

        for row in &self.skills {
            if let TuiRow::Item { item, .. } = row {
                if !item.included {
                    filter.toggle_skill(item.id.clone());
                }
            }
        }

        for row in &self.appimages {
            if let TuiRow::Item { item, .. } = row {
                if !item.included {
                    filter.toggle_appimage(item.id.clone());
                }
            }
        }

        for row in &self.zen_spaces {
            if let TuiRow::Item { item, .. } = row {
                if !item.included {
                    filter.toggle_space(item.id.clone());
                }
            }
        }

        for row in &self.ssh_keys {
            if let TuiRow::Item { item, .. } = row {
                if !item.included {
                    filter.toggle_ssh_key(item.id.clone());
                }
            }
        }

        filter
    }

    pub fn group_into_rows(groups: Vec<(String, String, Vec<TuiItem>)>) -> Vec<TuiRow> {
        let mut rows = Vec::new();
        for (cat_id, cat_title, items) in groups {
            if items.is_empty() {
                continue;
            }
            rows.push(TuiRow::Header {
                category_id: cat_id.clone(),
                title: cat_title,
                count: items.len(),
            });
            for item in items {
                rows.push(TuiRow::Item {
                    category_id: cat_id.clone(),
                    item,
                });
            }
        }
        rows
    }

    pub fn detect_staging_for_selected(&self) -> Option<PathBuf> {
        let snap = self.selected_snapshot()?;
        let target_manifest = snap.manifest.as_ref()?;

        if let Ok(user_home) = std::env::var("HOME") {
            let cache_dir = PathBuf::from(user_home).join(".cache");
            if let Some(found) = Self::find_matching_staging(&cache_dir, &target_manifest.snapshot_id) {
                return Some(found);
            }
        }

        if let Some(parent) = self.default_backups_dir.parent() {
            let cache_dir = parent.join(".cache");
            if let Some(found) = Self::find_matching_staging(&cache_dir, &target_manifest.snapshot_id) {
                return Some(found);
            }
        }

        None
    }

    fn find_matching_staging(cache_dir: &std::path::Path, expected_snapshot_id: &str) -> Option<PathBuf> {
        if let Ok(entries) = std::fs::read_dir(cache_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|s| s.starts_with(".tmp"))
                {
                    let manifest_file = p.join("manifest.json");
                    if manifest_file.exists() {
                        if let Ok(file) = std::fs::File::open(&manifest_file) {
                            if let Ok(m) = serde_json::from_reader::<_, Manifest>(file) {
                                if m.snapshot_id == expected_snapshot_id {
                                    return Some(p);
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}
