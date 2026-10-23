use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::entities::custom_directory::CustomDirectoryEntry;
use crate::value_objects::PackageName;

/// Declarative filter defining exclusions and user-selected custom directories
/// Configured either via CLI flags or interactively inside the Ratatui TUI
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionFilter {
    pub excluded_packages: HashSet<PackageName>,
    pub excluded_skills: HashSet<String>,
    pub excluded_appimages: HashSet<String>,
    pub excluded_spaces: HashSet<String>,
    pub custom_directories: Vec<CustomDirectoryEntry>,
}

impl SelectionFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_package_included(&self, package: &PackageName) -> bool {
        !self.excluded_packages.contains(package)
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
}
