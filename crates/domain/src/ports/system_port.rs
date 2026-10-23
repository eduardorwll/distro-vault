use crate::errors::DomainError;
use crate::value_objects::DistroFamily;

/// Hexagonal port interface for OS inspection, process management and runtime verification
pub trait SystemPort: Send + Sync {
    /// Identifies the running distribution family
    fn detect_distro(&self) -> Result<DistroFamily, DomainError>;

    /// Returns the local machine hostname
    fn hostname(&self) -> Result<String, DomainError>;

    /// Checks if a process with the given name is currently running
    fn is_process_running(&self, process_name: &str) -> bool;

    /// Attempts to terminate a process gracefully (SIGTERM), waiting up to timeout seconds
    fn terminate_process_gracefully(&self, process_name: &str, timeout_secs: u64) -> Result<(), DomainError>;

    /// Checks if libfuse2 is available for AppImage execution
    fn is_fuse2_available(&self) -> bool;
}
