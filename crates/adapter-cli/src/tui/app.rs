use std::path::PathBuf;

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

pub struct TuiApp {
    pub current_tab: usize,
    pub user_packages: Vec<TuiItem>,
    pub system_packages: Vec<TuiItem>,
    pub skills: Vec<TuiItem>,
    pub appimages: Vec<TuiItem>,
    pub custom_dirs: Vec<CustomDirectoryEntry>,
    pub zen_spaces: Vec<TuiItem>,
    pub cursors: [usize; 6],
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
            skills: Vec::new(),
            appimages: Vec::new(),
            custom_dirs: Vec::new(),
            zen_spaces: Vec::new(),
            cursors: [0; 6],
            input_mode: false,
            input_buffer: String::new(),
            status_message: None,
            should_quit: false,
            should_start_backup: false,
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 6;
    }

    pub fn prev_tab(&mut self) {
        if self.current_tab == 0 {
            self.current_tab = 5;
        } else {
            self.current_tab -= 1;
        }
    }

    pub fn current_list_len(&self) -> usize {
        match self.current_tab {
            0 => self.user_packages.len(),
            1 => self.system_packages.len(),
            2 => self.skills.len(),
            3 => self.appimages.len(),
            4 => self.custom_dirs.len(),
            5 => self.zen_spaces.len(),
            _ => 0,
        }
    }

    pub fn next_item(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            let cursor = &mut self.cursors[self.current_tab];
            *cursor = (*cursor + 1).min(len - 1);
        }
    }

    pub fn prev_item(&mut self) {
        let cursor = &mut self.cursors[self.current_tab];
        if *cursor > 0 {
            *cursor -= 1;
        }
    }

    pub fn toggle_selected(&mut self) {
        let idx = self.cursors[self.current_tab];
        match self.current_tab {
            0 => {
                if let Some(item) = self.user_packages.get_mut(idx) {
                    item.included = !item.included;
                }
            }
            1 => {
                if let Some(item) = self.system_packages.get_mut(idx) {
                    item.included = !item.included;
                }
            }
            2 => {
                if let Some(item) = self.skills.get_mut(idx) {
                    item.included = !item.included;
                }
            }
            3 => {
                if let Some(item) = self.appimages.get_mut(idx) {
                    item.included = !item.included;
                }
            }
            4 => {
                // If on custom dirs tab, pressing space removes the selected custom dir
                if idx < self.custom_dirs.len() {
                    self.custom_dirs.remove(idx);
                    if self.cursors[4] > 0 && self.cursors[4] >= self.custom_dirs.len() {
                        self.cursors[4] -= 1;
                    }
                }
            }
            5 => {
                if let Some(item) = self.zen_spaces.get_mut(idx) {
                    item.included = !item.included;
                }
            }
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

        for pkg in &self.user_packages {
            if !pkg.included {
                if let Ok(name) = PackageName::new(&pkg.id) {
                    filter.toggle_package(name);
                }
            }
        }

        for pkg in &self.system_packages {
            if !pkg.included {
                if let Ok(name) = PackageName::new(&pkg.id) {
                    filter.toggle_package(name);
                }
            }
        }

        for skill in &self.skills {
            if !skill.included {
                filter.toggle_skill(skill.id.clone());
            }
        }

        for app in &self.appimages {
            if !app.included {
                filter.toggle_appimage(app.id.clone());
            }
        }

        for space in &self.zen_spaces {
            if !space.included {
                filter.toggle_space(space.id.clone());
            }
        }

        filter
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}
