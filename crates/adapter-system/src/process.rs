use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use domain::errors::DomainError;

pub struct ProcessManager;

impl ProcessManager {
    /// Checks if a process with matching name is running by inspecting /proc
    pub fn is_running(process_name: &str) -> bool {
        let proc_dir = Path::new("/proc");
        if !proc_dir.exists() {
            return false;
        }

        if let Ok(entries) = fs::read_dir(proc_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                // Check comm file inside PID directories
                let comm_file = path.join("comm");
                if let Ok(comm) = fs::read_to_string(&comm_file) {
                    let comm_trimmed = comm.trim();
                    if comm_trimmed == process_name || comm_trimmed.contains(process_name) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Attempts to terminate a process gracefully using pkill SIGTERM, waiting up to timeout_secs
    pub fn terminate_gracefully(process_name: &str, timeout_secs: u64) -> Result<(), DomainError> {
        if !Self::is_running(process_name) {
            return Ok(());
        }

        let _ = std::process::Command::new("pkill")
            .arg("-TERM")
            .arg("-f")
            .arg(process_name)
            .status();

        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            if !Self::is_running(process_name) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(250));
        }

        if Self::is_running(process_name) {
            return Err(DomainError::BrowserError {
                message: format!(
                    "Process '{}' failed to terminate within {} seconds timeout",
                    process_name, timeout_secs
                ),
            });
        }

        Ok(())
    }
}
