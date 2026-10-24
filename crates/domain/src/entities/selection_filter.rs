use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::entities::custom_directory::CustomDirectoryEntry;
use crate::value_objects::PackageName;

fn default_true() -> bool {
    true
}

/// Declarative filter defining exclusions and user-selected custom directories
/// Configured either via CLI flags or interactively inside the Ratatui TUI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionFilter {
    pub excluded_packages: HashSet<PackageName>,
    #[serde(default)]
    pub included_dependencies: HashSet<PackageName>,
    pub excluded_skills: HashSet<String>,
    pub excluded_appimages: HashSet<String>,
    pub excluded_spaces: HashSet<String>,
    #[serde(default)]
    pub excluded_ssh_keys: HashSet<String>,
    pub custom_directories: Vec<CustomDirectoryEntry>,
    #[serde(default = "default_true")]
    pub restore_zen: bool,
    #[serde(default = "default_true")]
    pub restore_git_config: bool,
    #[serde(default)]
    pub excluded_custom_dirs: HashSet<String>,
}

impl Default for SelectionFilter {
    fn default() -> Self {
        Self {
            excluded_packages: HashSet::new(),
            included_dependencies: HashSet::new(),
            excluded_skills: HashSet::new(),
            excluded_appimages: HashSet::new(),
            excluded_spaces: HashSet::new(),
            excluded_ssh_keys: HashSet::new(),
            custom_directories: Vec::new(),
            restore_zen: true,
            restore_git_config: true,
            excluded_custom_dirs: HashSet::new(),
        }
    }
}

impl SelectionFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_package_included(&self, package: &PackageName) -> bool {
        !self.excluded_packages.contains(package)
    }

    pub fn is_dependency_included(&self, package: &PackageName) -> bool {
        self.included_dependencies.contains(package)
    }

    pub fn is_skill_included(&self, skill_name: &str) -> bool {
        !self.excluded_skills.contains(skill_name)
    }

    pub fn is_appimage_included(&self, appimage_name: &str) -> bool {
        !self.excluded_appimages.contains(appimage_name)
    }

    pub fn is_space_included(&self, space_id: &str) -> bool {
        !self.excluded_spaces.contains(space_id)
    }

    pub fn is_ssh_key_included(&self, key_name: &str) -> bool {
        !self.excluded_ssh_keys.contains(key_name)
    }

    pub fn exclude_package(&mut self, package: PackageName) {
        self.excluded_packages.insert(package);
    }

    pub fn include_package(&mut self, package: &PackageName) {
        self.excluded_packages.remove(package);
    }

    pub fn include_dependency(&mut self, package: PackageName) {
        self.included_dependencies.insert(package);
    }

    pub fn toggle_package(&mut self, package: PackageName) {
        if !self.excluded_packages.remove(&package) {
            self.excluded_packages.insert(package);
        }
    }

    pub fn toggle_skill(&mut self, skill_name: String) {
        if !self.excluded_skills.remove(&skill_name) {
            self.excluded_skills.insert(skill_name);
        }
    }

    pub fn toggle_appimage(&mut self, appimage_name: String) {
        if !self.excluded_appimages.remove(&appimage_name) {
            self.excluded_appimages.insert(appimage_name);
        }
    }

    pub fn toggle_space(&mut self, space_id: String) {
        if !self.excluded_spaces.remove(&space_id) {
            self.excluded_spaces.insert(space_id);
        }
    }

    pub fn toggle_ssh_key(&mut self, key_name: String) {
        if !self.excluded_ssh_keys.remove(&key_name) {
            self.excluded_ssh_keys.insert(key_name);
        }
    }

    pub fn exclude_ssh_key(&mut self, key_name: String) {
        self.excluded_ssh_keys.insert(key_name);
    }

    pub fn include_ssh_key(&mut self, key_name: &str) {
        self.excluded_ssh_keys.remove(key_name);
    }

    pub fn is_custom_dir_included(&self, dir_id_or_label: &str) -> bool {
        !self.excluded_custom_dirs.contains(dir_id_or_label)
    }

    pub fn toggle_custom_dir(&mut self, id: String) {
        if !self.excluded_custom_dirs.remove(&id) {
            self.excluded_custom_dirs.insert(id);
        }
    }

    pub fn set_restore_zen(&mut self, enabled: bool) {
        self.restore_zen = enabled;
    }

    pub fn set_restore_git_config(&mut self, enabled: bool) {
        self.restore_git_config = enabled;
    }
}
