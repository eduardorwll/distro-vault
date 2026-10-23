use std::path::Path;

use crate::entities::AgentGovernance;
use crate::errors::DomainError;

/// Hexagonal port interface for AI Agent Skills, Rules, and Governance
pub trait AgentPort: Send + Sync {
    /// Scans the user environment and models the active AgentGovernance entity
    fn scan_governance(&self, user_home: &Path) -> Result<AgentGovernance, DomainError>;

    /// Exports skills and rules, automatically dereferencing distro-specific symlinks
    fn export_governance(
        &self,
        governance: &AgentGovernance,
        dest_dir: &Path,
        dereference_symlinks: bool,
    ) -> Result<(), DomainError>;

    /// Restores skills, rules and quality gates into target environment
    fn restore_governance(&self, source_dir: &Path, target_home: &Path) -> Result<(), DomainError>;
}
