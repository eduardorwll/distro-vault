use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::value_objects::SkillCategory;

/// Definition of an agent rule file (e.g. AGENTS.md, GEMINI.md)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleEntry {
    pub name: String,
    pub relative_path: PathBuf,
    pub description: String,
    pub is_mandatory: bool,
}

/// Definition of an integrated AI Agent Skill
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillEntry {
    pub name: String,
    pub category: SkillCategory,
    pub is_symlink: bool,
    pub dereferenced: bool,
    pub original_target: Option<PathBuf>,
    pub relative_path: PathBuf,
    pub description: Option<String>,
}

/// Enforced engineering quality gates from the project governance paradigm
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityGates {
    pub mandatory_pre_approval: bool,
    pub five_phase_engineering_cycle: bool,
    pub zero_warning_clippy: bool,
    pub multi_agent_orchestration: bool,
    pub living_documentation_upkeep: bool,
}

impl Default for QualityGates {
    fn default() -> Self {
        Self {
            mandatory_pre_approval: true,
            five_phase_engineering_cycle: true,
            zero_warning_clippy: true,
            multi_agent_orchestration: true,
            living_documentation_upkeep: true,
        }
    }
}

/// First-class domain entity modeling the entire AI Agent Governance Ecosystem
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentGovernance {
    pub version: String,
    pub default_working_directory: String,
    pub rules: Vec<RuleEntry>,
    pub skills: Vec<SkillEntry>,
    pub quality_gates: QualityGates,
    pub cli_wrappers: Vec<String>,
}

impl AgentGovernance {
    pub fn new(default_working_directory: impl Into<String>) -> Self {
        Self {
            version: "1.0.0".to_string(),
            default_working_directory: default_working_directory.into(),
            rules: Vec::new(),
            skills: Vec::new(),
            quality_gates: QualityGates::default(),
            cli_wrappers: Vec::new(),
        }
    }

    pub fn skills_by_category(&self, category: SkillCategory) -> Vec<&SkillEntry> {
        self.skills.iter().filter(|s| s.category == category).collect()
    }
}
