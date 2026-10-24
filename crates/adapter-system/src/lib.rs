pub mod os_release;
pub mod packages;
pub mod process;
pub mod ssh;

use std::path::Path;

use domain::errors::DomainError;
use domain::ports::SystemPort;
use domain::value_objects::DistroFamily;

pub use os_release::OsReleaseParser;
pub use packages::{AptAdapter, AurAdapter, DnfAdapter, FlatpakAdapter, PacmanAdapter};
pub use process::ProcessManager;
pub use ssh::LinuxSshAdapter;

#[derive(Debug, Default, Clone)]
pub struct LinuxSystemAdapter;

impl LinuxSystemAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl SystemPort for LinuxSystemAdapter {
    fn detect_distro(&self) -> Result<DistroFamily, DomainError> {
        OsReleaseParser::parse_file(Path::new("/etc/os-release"))
    }

    fn hostname(&self) -> Result<String, DomainError> {
        OsReleaseParser::read_hostname()
    }

    fn is_process_running(&self, process_name: &str) -> bool {
        ProcessManager::is_running(process_name)
    }

    fn terminate_process_gracefully(&self, process_name: &str, timeout_secs: u64) -> Result<(), DomainError> {
        ProcessManager::terminate_gracefully(process_name, timeout_secs)
    }

    fn is_fuse2_available(&self) -> bool {
        OsReleaseParser::check_fuse2_installed()
    }
}
