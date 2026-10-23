use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

use adapter_agents::LocalAgentAdapter;
use adapter_browser::{MozLz4Codec, ZenBrowserAdapter};
use adapter_storage::{CustomDirsHandler, ZstdStorageAdapter};
use adapter_system::{LinuxSystemAdapter, PacmanAdapter};
use application::{
    BackupPipelineUseCase, BackupRequest, InspectSnapshotUseCase, RestorePipelineUseCase,
    RestoreRequest, VerifyIntegrityUseCase,
};
use domain::entities::selection_filter::SelectionFilter;
use domain::errors::DomainError;
use domain::ports::PackagePort;
use domain::value_objects::PackageName;

#[test]
fn test_full_backup_and_restore_pipeline() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let test_root = tmp.path();

    // 1. Setup mock environment
    let zen_config = test_root.join(".config/zen");
    let prof_dir = zen_config.join("7awtlvx8.Default (release)");
    fs::create_dir_all(&prof_dir).unwrap();

    let session_data = r#"{"spaces":[{"id":"s1","name":"DevSpace","tabs":["t1","t2"]}]}"#;
    let compressed_session = MozLz4Codec::compress(session_data).unwrap();
    fs::write(prof_dir.join("zen-sessions.jsonlz4"), compressed_session).unwrap();
    fs::write(prof_dir.join("places.sqlite"), b"SQLITE DB MOCK").unwrap();

    let user_home = test_root.join("home_user");
    let skills_dir = user_home.join(".agents/skills/clean-code");
    fs::create_dir_all(&skills_dir).unwrap();
    fs::write(
        skills_dir.join("SKILL.md"),
        "---\nname: clean-code\ndescription: Clean Code principles.\n---\nBody",
    ).unwrap();
    fs::write(user_home.join("GEMINI.md"), "# Guidelines\nDefault dir").unwrap();

    let custom_dir = test_root.join("my_docs");
    fs::create_dir_all(&custom_dir).unwrap();
    fs::write(custom_dir.join("project.md"), "# Secret Distro Project").unwrap();
    let custom_entry = CustomDirsHandler::inspect_directory(&custom_dir, Some("docs")).unwrap();

    // 2. Instantiate Adapters
    let browser = Arc::new(ZenBrowserAdapter::new());
    let storage = Arc::new(ZstdStorageAdapter::new());
    let agent = Arc::new(LocalAgentAdapter::new());
    let system = Arc::new(LinuxSystemAdapter::new());
    let pacman: Arc<dyn PackagePort> = Arc::new(PacmanAdapter::new());
    let package_ports = vec![pacman];

    // 3. Test BackupPipelineUseCase
    let backup_use_case = BackupPipelineUseCase::new(
        browser.clone(),
        storage.clone(),
        agent.clone(),
        system.clone(),
        package_ports.clone(),
    );

    let output_archive = test_root.join("system_snapshot.tar.zst");
    let mut filter = SelectionFilter::new();
    filter.toggle_skill("unwanted-skill".to_string());

    let backup_req = BackupRequest::new(&zen_config, &user_home, &output_archive)
        .with_filter(filter)
        .with_custom_directories(vec![custom_entry])
        .with_snapshot_id("snap-integration-01");

    let report = backup_use_case.execute(backup_req).expect("Backup execution failed");

    assert_eq!(report.snapshot_id, "snap-integration-01");
    assert!(output_archive.exists());
    assert_eq!(report.manifest.zen_profiles.len(), 1);
    assert_eq!(report.manifest.agent_governance.skills.len(), 1);
    assert_eq!(report.manifest.custom_directories.len(), 1);

    // 4. Test InspectSnapshotUseCase
    let inspect_use_case = InspectSnapshotUseCase::new(storage.clone());
    let summary = inspect_use_case.execute(&output_archive).expect("Inspect execution failed");

    assert_eq!(summary.snapshot_id, "snap-integration-01");
    assert_eq!(summary.zen_profiles_count, 1);
    assert_eq!(summary.spaces_count, 1);
    assert_eq!(summary.custom_dirs_count, 1);

    // 5. Test VerifyIntegrityUseCase
    let verify_use_case = VerifyIntegrityUseCase::new(storage.clone());
    let integrity = verify_use_case.execute(&output_archive).expect("Verify execution failed");

    assert!(integrity.is_valid);
    assert_eq!(integrity.computed_checksum, report.checksum);

    // 6. Test RestorePipelineUseCase
    let restore_target_config = test_root.join("restored_config/zen");
    let restore_target_home = test_root.join("restored_home");
    let restore_target_custom = test_root.join("restored_custom");

    let restore_use_case = RestorePipelineUseCase::new(
        browser,
        storage,
        agent,
        system,
        package_ports,
    );

    let restore_req = RestoreRequest::new(&output_archive, &restore_target_config, &restore_target_home)
        .with_custom_base(&restore_target_custom)
        .with_packages(false, false);

    let restore_report = restore_use_case.execute(restore_req).expect("Restore execution failed");

    assert_eq!(restore_report.snapshot_id, "snap-integration-01");
    assert_eq!(restore_report.profiles_restored.len(), 1);
    assert_eq!(restore_report.skills_restored, 1);

    // Verify restored browser profile & normalized ini files
    let restored_prof = restore_target_config.join("7awtlvx8.Default (release)");
    assert!(restored_prof.exists());
    assert!(restored_prof.join("zen-sessions.jsonlz4").exists());
    assert!(restore_target_config.join("installs.ini").exists());
    assert!(restore_target_config.join("profiles.ini").exists());

    // Verify restored skills
    let restored_skill = restore_target_home.join(".agents/skills/clean-code/SKILL.md");
    assert!(restored_skill.exists());

    // Verify restored custom directories
    let restored_custom_file = restore_target_custom.join("my_docs/project.md");
    assert!(restored_custom_file.exists());
    assert_eq!(fs::read_to_string(restored_custom_file).unwrap(), "# Secret Distro Project");
}

#[test]
fn test_backup_aborts_when_browser_locked() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let zen_config = tmp.path().join(".config/zen");
    let prof_dir = zen_config.join("test.profile");
    fs::create_dir_all(&prof_dir).unwrap();
    // Simulate active .parentlock file
    fs::write(zen_config.join(".parentlock"), b"").unwrap();

    let user_home = tmp.path().join("home_user");
    fs::create_dir_all(&user_home).unwrap();
    let output_archive = tmp.path().join("out.tar.zst");

    let backup_use_case = BackupPipelineUseCase::new(
        Arc::new(ZenBrowserAdapter::new()),
        Arc::new(ZstdStorageAdapter::new()),
        Arc::new(LocalAgentAdapter::new()),
        Arc::new(LinuxSystemAdapter::new()),
        vec![],
    );

    let req = BackupRequest::new(&zen_config, &user_home, &output_archive);
    let result = backup_use_case.execute(req);

    assert!(result.is_err());
    match result.unwrap_err() {
        DomainError::BrowserLocked { lock_file } => {
            assert!(lock_file.to_string_lossy().contains(".parentlock"));
        }
        err => panic!("Expected BrowserLocked, got {:?}", err),
    }
}

#[test]
fn test_selection_filter_excludes_items_deterministically() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let zen_config = tmp.path().join(".config/zen");
    let user_home = tmp.path().join("home_user");
    let skills_dir = user_home.join(".agents/skills");
    fs::create_dir_all(skills_dir.join("clean-code")).unwrap();
    fs::create_dir_all(skills_dir.join("unwanted-skill")).unwrap();

    let backup_use_case = BackupPipelineUseCase::new(
        Arc::new(ZenBrowserAdapter::new()),
        Arc::new(ZstdStorageAdapter::new()),
        Arc::new(LocalAgentAdapter::new()),
        Arc::new(LinuxSystemAdapter::new()),
        vec![],
    );

    let output_archive = tmp.path().join("filtered.tar.zst");
    let mut filter = SelectionFilter::new();
    filter.toggle_skill("unwanted-skill".to_string());
    filter.toggle_package(PackageName::new("ripgrep").unwrap());

    let req = BackupRequest::new(&zen_config, &user_home, &output_archive).with_filter(filter);
    let report = backup_use_case.execute(req).expect("Backup failed");

    // "unwanted-skill" should NOT be in manifest
    assert!(!report.manifest.agent_governance.skills.iter().any(|s| s.name == "unwanted-skill"));
    assert!(report.manifest.agent_governance.skills.iter().any(|s| s.name == "clean-code"));
}
