use std::path::{Path, PathBuf};

/// State for the clean Execution Parameters & Options Modal (anti-bloat)
#[derive(Debug, Clone, Default)]
pub struct OptionsModalState {
    pub is_open: bool,
}

impl OptionsModalState {
    pub fn new() -> Self {
        Self { is_open: false }
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    OutputDir,
    InputDirsList,
    CompactViewToggle,
    SaveAndClose,
}

/// State for the I/O Directories & Settings Modal
#[derive(Debug, Clone)]
pub struct ConfigModalState {
    pub is_open: bool,
    pub focused_field: ConfigField,
    pub input_dir_cursor: usize,
    pub pending_input_dirs: Vec<PathBuf>,
    pub pending_output_dir: PathBuf,
    pub pending_compact_view: bool,
}

impl Default for ConfigModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            focused_field: ConfigField::OutputDir,
            input_dir_cursor: 0,
            pending_input_dirs: Vec::new(),
            pending_output_dir: PathBuf::from("./backups"),
            pending_compact_view: false,
        }
    }
}

impl ConfigModalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(
        &mut self,
        input_dirs: &[PathBuf],
        output_dir: &Path,
        compact_view: bool,
    ) {
        self.is_open = true;
        self.focused_field = ConfigField::OutputDir;
        self.input_dir_cursor = 0;
        self.pending_input_dirs = input_dirs.to_vec();
        self.pending_output_dir = output_dir.to_path_buf();
        self.pending_compact_view = compact_view;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            ConfigField::OutputDir => ConfigField::InputDirsList,
            ConfigField::InputDirsList => ConfigField::CompactViewToggle,
            ConfigField::CompactViewToggle => ConfigField::SaveAndClose,
            ConfigField::SaveAndClose => ConfigField::OutputDir,
        };
    }

    pub fn prev_field(&mut self) {
        self.focused_field = match self.focused_field {
            ConfigField::OutputDir => ConfigField::SaveAndClose,
            ConfigField::InputDirsList => ConfigField::OutputDir,
            ConfigField::CompactViewToggle => ConfigField::InputDirsList,
            ConfigField::SaveAndClose => ConfigField::CompactViewToggle,
        };
    }

    pub fn next_input_dir(&mut self) {
        if !self.pending_input_dirs.is_empty()
            && self.input_dir_cursor + 1 < self.pending_input_dirs.len()
        {
            self.input_dir_cursor += 1;
        }
    }

    pub fn prev_input_dir(&mut self) {
        if self.input_dir_cursor > 0 {
            self.input_dir_cursor -= 1;
        }
    }

    pub fn remove_selected_input_dir(&mut self) {
        if !self.pending_input_dirs.is_empty()
            && self.input_dir_cursor < self.pending_input_dirs.len()
        {
            self.pending_input_dirs.remove(self.input_dir_cursor);
            if self.input_dir_cursor >= self.pending_input_dirs.len()
                && self.input_dir_cursor > 0
            {
                self.input_dir_cursor -= 1;
            }
        }
    }

    pub fn add_input_dir(&mut self, dir: PathBuf) {
        if !self.pending_input_dirs.contains(&dir) {
            self.pending_input_dirs.push(dir);
        }
    }

    pub fn set_output_dir(&mut self, dir: PathBuf) {
        if !self.pending_input_dirs.contains(&dir) {
            self.pending_input_dirs.push(dir.clone());
        }
        self.pending_output_dir = dir;
    }

    pub fn toggle_compact_view(&mut self) {
        self.pending_compact_view = !self.pending_compact_view;
    }
}
