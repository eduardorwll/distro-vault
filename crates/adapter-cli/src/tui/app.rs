use std::path::PathBuf;
use ratatui::widgets::ListState;

use adapter_storage::CustomDirsHandler;
use domain::entities::custom_directory::CustomDirectoryEntry;
use domain::entities::selection_filter::SelectionFilter;
use domain::value_objects::PackageName;

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
    pub current_tab: usize,
    pub user_packages: Vec<TuiRow>,
    pub system_packages: Vec<TuiRow>,
    pub dependency_packages: Vec<TuiRow>,
    pub skills: Vec<TuiRow>,
    pub appimages: Vec<TuiRow>,
    pub custom_dirs: Vec<CustomDirectoryEntry>,
    pub zen_spaces: Vec<TuiRow>,
    pub cursors: [usize; 7],
    pub list_states: [ListState; 7],
    pub input_mode: bool,
    pub input_buffer: String,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub should_start_backup: bool,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            current_tab: 0,
            user_packages: Vec::new(),
            system_packages: Vec::new(),
            dependency_packages: Vec::new(),
            skills: Vec::new(),
            appimages: Vec::new(),
            custom_dirs: Vec::new(),
            zen_spaces: Vec::new(),
            cursors: [0; 7],
            list_states: [
                ListState::default(),
                ListState::default(),
                ListState::default(),
                ListState::default(),
                ListState::default(),
                ListState::default(),
                ListState::default(),
            ],
            input_mode: false,
            input_buffer: String::new(),
            status_message: None,
            should_quit: false,
            should_start_backup: false,
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 7;
    }

    pub fn prev_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = 6;
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
                            TuiRow::Item { category_id: c, item } if c == &cat => item.included,
                            _ => false,
                        });
                        let new_state = !any_included;
                        for r in rows.iter_mut() {
                            if let TuiRow::Item { category_id: c, item } = r {
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
            _ => {}
        }
    }

    pub fn add_custom_directory(&mut self, path_str: &str) -> Result<(), String> {
        let path = PathBuf::from(path_str.trim());
        let inspected = CustomDirsHandler::inspect_directory(&path, None)
            .map_err(|e| format!("{}", e))?;

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
                            filt.toggle_package(name);
                        }
                    }
                }
            }
        };

        scan_packages(&self.user_packages, &mut filter);
        scan_packages(&self.system_packages, &mut filter);
        scan_packages(&self.dependency_packages, &mut filter);

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
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}
