pub mod scanner;

use std::fs;
use std::path::Path;

use domain::entities::AgentGovernance;
use domain::errors::DomainError;
use domain::ports::AgentPort;

pub use scanner::AgentScanner;

#[derive(Debug, Default, Clone)]
pub struct LocalAgentAdapter;

impl LocalAgentAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl AgentPort for LocalAgentAdapter {
    fn scan_governance(&self, user_home: &Path) -> Result<AgentGovernance, DomainError> {
        AgentScanner::scan(user_home)
    }

    fn export_governance(
        &self,
        governance: &AgentGovernance,
        dest_dir: &Path,
        dereference_symlinks: bool,
    ) -> Result<(), DomainError> {
        // Export rules
        for rule in &governance.rules {
            let src = dest_dir.join(&rule.relative_path);
            let target_dest = dest_dir.join(&rule.relative_path);
            if src.exists() && src != target_dest {
                if let Some(parent) = target_dest.parent() {
                    fs::create_dir_all(parent).map_err(|e| DomainError::AgentGovernanceError {
                        message: format!("Failed to create parent dir {:?}: {}", parent, e),
                    })?;
                }
                fs::copy(&src, &target_dest).map_err(|e| DomainError::AgentGovernanceError {
                    message: format!("Failed to export rule {:?}: {}", src, e),
                })?;
            }
        }

        // Export skills
        for skill in &governance.skills {
            let skill_dest = dest_dir.join(&skill.relative_path);

            if let Some(ref target) = skill.original_target {
                if dereference_symlinks {
                    AgentScanner::copy_dereferenced(target, &skill_dest)?;
                }
            }
        }

        Ok(())
    }

    fn restore_governance(&self, source_dir: &Path, target_home: &Path) -> Result<(), DomainError> {
        if !source_dir.exists() {
            return Ok(());
        }

        AgentScanner::copy_dereferenced(source_dir, target_home)
    }
}
