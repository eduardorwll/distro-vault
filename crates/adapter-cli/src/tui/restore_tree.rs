use ratatui::widgets::ListState;

use domain::entities::manifest::Manifest;
use domain::entities::selection_filter::SelectionFilter;
use domain::value_objects::PackageName;

/// Kinds of dump categories that can be selectively restored
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreCategoryKind {
    AppsZen,
    SshKeys,
    Packages,
    CustomDirs,
    GitConfig,
    Skills,
}

/// A leaf item inside a category (e.g., specific SSH key, specific package, specific directory)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreItemNode {
    pub id: String,
    pub label: String,
    pub sublabel: Option<String>,
    pub is_selected: bool,
}

/// A top-level category in the dump tree that can be expanded/collapsed (drop menu) and selected in batch
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreCategoryNode {
    pub kind: RestoreCategoryKind,
    pub title: String,
    pub icon: &'static str,
    pub is_expanded: bool,
    pub is_selected: bool,
    pub items: Vec<RestoreItemNode>,
}

impl RestoreCategoryNode {
    pub fn new(
        kind: RestoreCategoryKind,
        title: impl Into<String>,
        icon: &'static str,
        items: Vec<RestoreItemNode>,
    ) -> Self {
        Self {
            kind,
            title: title.into(),
            icon,
            is_expanded: false,
            is_selected: true,
            items,
        }
    }

    pub fn selected_count(&self) -> usize {
        self.items.iter().filter(|i| i.is_selected).count()
    }
}

/// Node visible in the flattened tree for UI rendering and navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibleTreeNode {
    Category(usize),
    Item(usize, usize),
}

/// State for the interactive granular restore selector tree with drop/collapse menus
#[derive(Debug, Clone, Default)]
pub struct RestoreTreeState {
    pub categories: Vec<RestoreCategoryNode>,
    pub cursor: usize,
    pub list_state: ListState,
}

impl RestoreTreeState {
    /// Builds a granular tree from the inspected snapshot manifest
    pub fn from_manifest(manifest: &Manifest) -> Self {
        let mut categories = Vec::new();

        // 1. Apps & Zen Browser Spaces
        let mut app_items = Vec::new();
        for prof in &manifest.zen_profiles {
            for space in &prof.spaces {
                app_items.push(RestoreItemNode {
                    id: format!("{}:{}", prof.id.as_str(), space.id),
                    label: format!("Espaço Zen: {}", space.name),
                    sublabel: Some(format!("{} aba(s) abertas", space.open_tabs_count)),
                    is_selected: true,
                });
            }
            if prof.spaces.is_empty() {
                app_items.push(RestoreItemNode {
                    id: prof.id.as_str().to_string(),
                    label: format!("Perfil Zen: {}", prof.id.as_str()),
                    sublabel: Some(format!("Hash/Path: {}", prof.id.as_str())),
                    is_selected: true,
                });
            }
        }
        for app in &manifest.appimages {
            app_items.push(RestoreItemNode {
                id: app.name.clone(),
                label: format!("AppImage: {}", app.name),
                sublabel: Some(format!("{:.1} MB", app.size_bytes as f64 / 1024.0 / 1024.0)),
                is_selected: true,
            });
        }
        if !app_items.is_empty() {
            categories.push(RestoreCategoryNode::new(
                RestoreCategoryKind::AppsZen,
                format!("Navegador & Apps ({} itens)", app_items.len()),
                "🌐",
                app_items,
            ));
        }

        // 2. SSH Keys
        let mut ssh_items = Vec::new();
        for key in &manifest.ssh_keys {
            ssh_items.push(RestoreItemNode {
                id: key.file_name.clone(),
                label: key.file_name.clone(),
                sublabel: Some(format!(
                    "{} (modo {:04o}, {} B)",
                    key.file_type.as_str(),
                    key.posix_mode,
                    key.size_bytes
                )),
                is_selected: true,
            });
        }
        if !ssh_items.is_empty() {
            categories.push(RestoreCategoryNode::new(
                RestoreCategoryKind::SshKeys,
                format!("Chaves SSH (~/.ssh) ({} chaves/configs)", ssh_items.len()),
                "🔑",
                ssh_items,
            ));
        }

        // 3. Custom Directories
        let mut dir_items = Vec::new();
        for dir in &manifest.custom_directories {
            dir_items.push(RestoreItemNode {
                id: dir.id.clone(),
                label: dir.source_path.display().to_string(),
                sublabel: Some(format!(
                    "{} ({} arquivos, {:.1} MB)",
                    dir.label,
                    dir.file_count,
                    dir.size_bytes as f64 / 1024.0 / 1024.0
                )),
                is_selected: true,
            });
        }
        if !dir_items.is_empty() {
            categories.push(RestoreCategoryNode::new(
                RestoreCategoryKind::CustomDirs,
                format!("Pastas Customizadas ({} diretórios)", dir_items.len()),
                "📁",
                dir_items,
            ));
        }

        // 4. Git Global Configuration
        let mut git_items = Vec::new();
        for git_cfg in &manifest.git_configs {
            let author = match (&git_cfg.user_name, &git_cfg.user_email) {
                (Some(n), Some(e)) => format!("{} <{}>", n, e),
                (Some(n), None) => n.clone(),
                (None, Some(e)) => e.clone(),
                (None, None) => "Configuração global".to_string(),
            };
            git_items.push(RestoreItemNode {
                id: git_cfg.relative_path.clone(),
                label: git_cfg.file_name.clone(),
                sublabel: Some(author),
                is_selected: true,
            });
        }
        if !git_items.is_empty() {
            categories.push(RestoreCategoryNode::new(
                RestoreCategoryKind::GitConfig,
                format!("Configuração Global Git ({} arquivos)", git_items.len()),
                "👤",
                git_items,
            ));
        }

        // 5. Skills & AI Agents
        let mut skill_items = Vec::new();
        for skill in &manifest.agent_governance.skills {
            skill_items.push(RestoreItemNode {
                id: skill.name.clone(),
                label: skill.name.clone(),
                sublabel: skill.description.clone(),
                is_selected: true,
            });
        }
        if !skill_items.is_empty() {
            categories.push(RestoreCategoryNode::new(
                RestoreCategoryKind::Skills,
                format!("Skills & Agentes de IA ({} skills)", skill_items.len()),
                "🤖",
                skill_items,
            ));
        }

        // 6. Packages
        let mut pkg_items = Vec::new();
        for pkg in &manifest.packages {
            pkg_items.push(RestoreItemNode {
                id: pkg.name.as_str().to_string(),
                label: pkg.name.as_str().to_string(),
                sublabel: Some(format!("{:?} - {:?}", pkg.source, pkg.scope)),
                is_selected: true,
            });
        }
        if !pkg_items.is_empty() {
            categories.push(RestoreCategoryNode::new(
                RestoreCategoryKind::Packages,
                format!("Pacotes Declarativos ({} pacotes)", pkg_items.len()),
                "📦",
                pkg_items,
            ));
        }

        let mut state = Self {
            categories,
            cursor: 0,
            list_state: ListState::default(),
        };
        state.list_state.select(Some(0));
        state
    }

    /// Computes flattened list of visible nodes according to category expansion state
    pub fn visible_nodes(&self) -> Vec<VisibleTreeNode> {
        let mut nodes = Vec::new();
        for (cat_idx, cat) in self.categories.iter().enumerate() {
            nodes.push(VisibleTreeNode::Category(cat_idx));
            if cat.is_expanded {
                for item_idx in 0..cat.items.len() {
                    nodes.push(VisibleTreeNode::Item(cat_idx, item_idx));
                }
            }
        }
        nodes
    }

    pub fn current_visible(&self) -> Option<VisibleTreeNode> {
        let nodes = self.visible_nodes();
        nodes.get(self.cursor).copied()
    }

    pub fn next(&mut self) {
        let count = self.visible_nodes().len();
        if count == 0 {
            return;
        }
        if self.cursor + 1 < count {
            self.cursor += 1;
        } else {
            self.cursor = 0;
        }
        self.list_state.select(Some(self.cursor));
    }

    pub fn prev(&mut self) {
        let count = self.visible_nodes().len();
        if count == 0 {
            return;
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        } else {
            self.cursor = count - 1;
        }
        self.list_state.select(Some(self.cursor));
    }

    /// Toggles drop menu (expand/collapse) on the current category
    pub fn toggle_expanded(&mut self) {
        if let Some(visible) = self.current_visible() {
            match visible {
                VisibleTreeNode::Category(cat_idx) => {
                    if let Some(cat) = self.categories.get_mut(cat_idx) {
                        cat.is_expanded = !cat.is_expanded;
                    }
                }
                VisibleTreeNode::Item(cat_idx, _) => {
                    // Collapses parent category if triggered while on an item
                    if let Some(cat) = self.categories.get_mut(cat_idx) {
                        cat.is_expanded = false;
                        // Put cursor back on the category node
                        let nodes = self.visible_nodes();
                        if let Some(pos) = nodes.iter().position(|n| *n == VisibleTreeNode::Category(cat_idx)) {
                            self.cursor = pos;
                            self.list_state.select(Some(self.cursor));
                        }
                    }
                }
            }
        }
    }

    /// Toggles selection of category (in batch) or individual item
    pub fn toggle_selected(&mut self) {
        if let Some(visible) = self.current_visible() {
            match visible {
                VisibleTreeNode::Category(cat_idx) => {
                    if let Some(cat) = self.categories.get_mut(cat_idx) {
                        let new_state = !cat.is_selected;
                        cat.is_selected = new_state;
                        for item in &mut cat.items {
                            item.is_selected = new_state;
                        }
                    }
                }
                VisibleTreeNode::Item(cat_idx, item_idx) => {
                    if let Some(cat) = self.categories.get_mut(cat_idx) {
                        if let Some(item) = cat.items.get_mut(item_idx) {
                            item.is_selected = !item.is_selected;
                        }
                        // Update category parent state
                        let any_selected = cat.items.iter().any(|i| i.is_selected);
                        cat.is_selected = any_selected;
                    }
                }
            }
        }
    }

    /// Builds a concrete `SelectionFilter` reflecting the user's granular checkboxes
    pub fn build_selection_filter(&self) -> SelectionFilter {
        let mut filter = SelectionFilter::new();

        for cat in &self.categories {
            match cat.kind {
                RestoreCategoryKind::AppsZen => {
                    filter.set_restore_zen(cat.is_selected);
                    for item in &cat.items {
                        if !item.is_selected {
                            if item.id.starts_with("Espaço") || item.id.contains(':') {
                                filter.toggle_space(item.id.clone());
                            } else {
                                filter.toggle_appimage(item.id.clone());
                            }
                        }
                    }
                }
                RestoreCategoryKind::SshKeys => {
                    for item in &cat.items {
                        if !item.is_selected {
                            filter.exclude_ssh_key(item.id.clone());
                        }
                    }
                }
                RestoreCategoryKind::CustomDirs => {
                    for item in &cat.items {
                        if !item.is_selected {
                            filter.toggle_custom_dir(item.id.clone());
                        }
                    }
                }
                RestoreCategoryKind::GitConfig => {
                    filter.set_restore_git_config(cat.is_selected);
                }
                RestoreCategoryKind::Skills => {
                    for item in &cat.items {
                        if !item.is_selected {
                            filter.toggle_skill(item.id.clone());
                        }
                    }
                }
                RestoreCategoryKind::Packages => {
                    for item in &cat.items {
                        if !item.is_selected {
                            if let Ok(pkg_name) = PackageName::new(&item.id) {
                                filter.exclude_package(pkg_name);
                            }
                        }
                    }
                }
            }
        }

        filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::entities::agent_governance::AgentGovernance;
    use domain::entities::ssh_key::{SshFileType, SshKeyEntry};
    use domain::value_objects::DistroFamily;
    use std::path::PathBuf;

    #[test]
    fn test_restore_tree_expansion_and_selection() {
        let mut manifest = Manifest::new(
            "test-snap",
            "2026-10-24T18:00:00Z",
            "host",
            DistroFamily::Arch,
            AgentGovernance::new("/home/user"),
        );

        manifest.ssh_keys.push(SshKeyEntry::new(
            "id_ed25519",
            PathBuf::from("id_ed25519"),
            SshFileType::PrivateKey,
            411,
            0o600,
        ));
        manifest.ssh_keys.push(SshKeyEntry::new(
            "id_ed25519.pub",
            PathBuf::from("id_ed25519.pub"),
            SshFileType::PublicKey,
            103,
            0o644,
        ));

        let mut tree = RestoreTreeState::from_manifest(&manifest);
        assert_eq!(tree.categories.len(), 1);
        assert_eq!(tree.visible_nodes().len(), 1); // Collapsed by default

        // Expand drop menu
        tree.toggle_expanded();
        assert_eq!(tree.visible_nodes().len(), 3); // 1 Category + 2 child items

        // Navigate to first child item and uncheck it
        tree.next();
        assert_eq!(tree.cursor, 1);
        tree.toggle_selected();

        let filter = tree.build_selection_filter();
        assert!(!filter.is_ssh_key_included("id_ed25519"));
        assert!(filter.is_ssh_key_included("id_ed25519.pub"));

        // Collapse drop menu
        tree.toggle_expanded();
        assert_eq!(tree.visible_nodes().len(), 1);
    }
}
