use std::sync::Arc;

use adapter_agents::LocalAgentAdapter;
use adapter_browser::ZenBrowserAdapter;
use adapter_storage::ZstdStorageAdapter;
use adapter_system::{AurAdapter, FlatpakAdapter, LinuxSystemAdapter, PacmanAdapter};
use application::{
    BackupPipelineUseCase, InspectSnapshotUseCase, RestorePipelineUseCase, VerifyIntegrityUseCase,
};
use domain::ports::PackagePort;

/// Composition Root aggregating all wired use cases and concrete infrastructure adapters
pub struct AppContainer {
    pub backup_use_case: BackupPipelineUseCase,
    pub restore_use_case: RestorePipelineUseCase,
    pub inspect_use_case: InspectSnapshotUseCase,
    pub verify_use_case: VerifyIntegrityUseCase,

    pub browser_adapter: Arc<ZenBrowserAdapter>,
    pub storage_adapter: Arc<ZstdStorageAdapter>,
    pub agent_adapter: Arc<LocalAgentAdapter>,
    pub system_adapter: Arc<LinuxSystemAdapter>,
    pub package_ports: Vec<Arc<dyn PackagePort>>,
}

impl AppContainer {
    pub fn bootstrap() -> Self {
        let browser = Arc::new(ZenBrowserAdapter::new());
        let storage = Arc::new(ZstdStorageAdapter::new());
        let agent = Arc::new(LocalAgentAdapter::new());
        let system = Arc::new(LinuxSystemAdapter::new());

        let pacman: Arc<dyn PackagePort> = Arc::new(PacmanAdapter::new());
        let aur: Arc<dyn PackagePort> = Arc::new(AurAdapter::new());
        let flatpak: Arc<dyn PackagePort> = Arc::new(FlatpakAdapter::new());
        let package_ports = vec![pacman, aur, flatpak];

        let backup_use_case = BackupPipelineUseCase::new(
            browser.clone(),
            storage.clone(),
            agent.clone(),
            system.clone(),
            package_ports.clone(),
        );

        let restore_use_case = RestorePipelineUseCase::new(
            browser.clone(),
            storage.clone(),
            agent.clone(),
            system.clone(),
            package_ports.clone(),
        );

        let inspect_use_case = InspectSnapshotUseCase::new(storage.clone());
        let verify_use_case = VerifyIntegrityUseCase::new(storage.clone());

        Self {
            backup_use_case,
            restore_use_case,
            inspect_use_case,
            verify_use_case,
            browser_adapter: browser,
            storage_adapter: storage,
            agent_adapter: agent,
            system_adapter: system,
            package_ports,
        }
    }
}
