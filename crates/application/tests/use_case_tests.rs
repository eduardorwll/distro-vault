use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use tempfile::tempdir;

use adapter_agents::LocalAgentAdapter;
use adapter_browser::{MozLz4Codec, ZenBrowserAdapter};
use adapter_storage::{CustomDirsHandler, ZstdStorageAdapter};
use adapter_system::{LinuxGitConfigAdapter, LinuxSshAdapter, LinuxSystemAdapter, PacmanAdapter};
use application::{
    BackupPipelineUseCase, BackupRequest, InspectSnapshotUseCase, RestorePipelineUseCase,
    RestoreRequest, SliceSnapshotRequest, SliceSnapshotUseCase, VerifyIntegrityUseCase,
};
use domain::entities::selection_filter::SelectionFilter;
use domain::errors::DomainError;
use domain::ports::{PackagePort, StoragePort};
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
    )
    .unwrap();
    fs::write(user_home.join("GEMINI.md"), "# Guidelines\nDefault dir").unwrap();

    let ssh_dir = user_home.join(".ssh");
    fs::create_dir_all(&ssh_dir).unwrap();
    fs::write(
        ssh_dir.join("id_ed25519"),
        "-----BEGIN OPENSSH PRIVATE KEY-----\nMOCK_KEY_DATA\n-----END OPENSSH PRIVATE KEY-----",
    )
    .unwrap();
    fs::write(
        ssh_dir.join("id_ed25519.pub"),
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5 mock@test",
    )
    .unwrap();
    fs::write(ssh_dir.join("config"), "Host github.com\n  User git\n").unwrap();

    // Global Git configuration
    fs::write(
        user_home.join(".gitconfig"),
        "[user]\n  name = Test User\n  email = test@example.com\n",
    )
    .unwrap();

    let custom_dir = test_root.join("my_docs");
    fs::create_dir_all(&custom_dir).unwrap();
    fs::write(custom_dir.join("project.md"), "# Secret Distro Project").unwrap();
    let custom_entry = CustomDirsHandler::inspect_directory(&custom_dir, Some("docs")).unwrap();

    // 2. Instantiate Adapters
    let browser = Arc::new(ZenBrowserAdapter::new());
    let storage = Arc::new(ZstdStorageAdapter::new());
    let agent = Arc::new(LocalAgentAdapter::new());
    let system = Arc::new(LinuxSystemAdapter::new());
    let ssh = Arc::new(LinuxSshAdapter::new());
    let git_config = Arc::new(LinuxGitConfigAdapter::new());
    let pacman: Arc<dyn PackagePort> = Arc::new(PacmanAdapter::new());
    let package_ports = vec![pacman];

    // 3. Test BackupPipelineUseCase
    let backup_use_case = BackupPipelineUseCase::new(
        browser.clone(),
        storage.clone(),
        agent.clone(),
        system.clone(),
        ssh.clone(),
        git_config.clone(),
        package_ports.clone(),
    );

    let output_archive = test_root.join("system_snapshot.tar.zst");
    let mut filter = SelectionFilter::new();
    filter.toggle_skill("unwanted-skill".to_string());

    let backup_req = BackupRequest::new(&zen_config, &user_home, &output_archive)
        .with_filter(filter)
        .with_custom_directories(vec![custom_entry])
        .with_snapshot_id("snap-integration-01");

    let report = backup_use_case
        .execute(backup_req)
        .expect("Backup execution failed");

    assert_eq!(report.snapshot_id, "snap-integration-01");
    assert!(output_archive.exists());
    assert_eq!(report.manifest.zen_profiles.len(), 1);
    assert_eq!(report.manifest.agent_governance.skills.len(), 1);
    assert_eq!(report.manifest.custom_directories.len(), 1);

    // 4. Test InspectSnapshotUseCase
    let inspect_use_case = InspectSnapshotUseCase::new(storage.clone());
    let summary = inspect_use_case
        .execute(&output_archive)
        .expect("Inspect execution failed");

    assert_eq!(summary.snapshot_id, "snap-integration-01");
    assert_eq!(summary.zen_profiles_count, 1);
    assert_eq!(summary.spaces_count, 1);
    assert_eq!(summary.custom_dirs_count, 1);
    assert_eq!(summary.ssh_keys_count, 3);

    // 5. Test VerifyIntegrityUseCase
    let verify_use_case = VerifyIntegrityUseCase::new(storage.clone());
    let integrity = verify_use_case
        .execute(&output_archive)
        .expect("Verify execution failed");

    assert!(integrity.is_valid);
    assert_eq!(integrity.computed_checksum, report.checksum);

    // 6. Test RestorePipelineUseCase
    let restore_target_config = test_root.join("restored_config/zen");
    let restore_target_home = test_root.join("restored_home");
    let restore_target_custom = test_root.join("restored_custom");

    let restore_use_case =
        RestorePipelineUseCase::new(browser, storage, agent, system, ssh, git_config, package_ports);

    let restore_req = RestoreRequest::new(
        &output_archive,
        &restore_target_config,
        &restore_target_home,
    )
    .with_custom_base(&restore_target_custom)
    .with_packages(false, false);

    let restore_report = restore_use_case
        .execute(restore_req)
        .expect("Restore execution failed");

    assert_eq!(restore_report.snapshot_id, "snap-integration-01");
    assert_eq!(restore_report.profiles_restored.len(), 1);
    assert_eq!(restore_report.skills_restored, 1);
    assert_eq!(restore_report.ssh_keys_restored, 3);
    assert_eq!(restore_report.git_configs_restored, 1);
    assert!(restore_target_home.join(".gitconfig").exists());
    let restored_git_content = fs::read_to_string(restore_target_home.join(".gitconfig")).unwrap();
    assert!(restored_git_content.contains("name = Test User"));

    // Verify restored browser profile & normalized ini files
    let restored_prof = restore_target_config.join("7awtlvx8.Default (release)");
    assert!(restored_prof.exists());
    assert!(restored_prof.join("zen-sessions.jsonlz4").exists());
    assert!(restore_target_config.join("installs.ini").exists());
    assert!(restore_target_config.join("profiles.ini").exists());

    // Verify restored skills
    let restored_skill = restore_target_home.join(".agents/skills/clean-code/SKILL.md");
    assert!(restored_skill.exists());

    // Verify restored SSH keys with strict POSIX permissions
    let restored_ssh = restore_target_home.join(".ssh");
    assert!(restored_ssh.exists());
    assert_eq!(
        fs::metadata(&restored_ssh).unwrap().permissions().mode() & 0o777,
        0o700
    );

    let restored_priv = restored_ssh.join("id_ed25519");
    assert!(restored_priv.exists());
    assert_eq!(
        fs::metadata(&restored_priv).unwrap().permissions().mode() & 0o777,
        0o600
    );

    let restored_pub = restored_ssh.join("id_ed25519.pub");
    assert!(restored_pub.exists());
    assert_eq!(
        fs::metadata(&restored_pub).unwrap().permissions().mode() & 0o777,
        0o644
    );

    let restored_conf = restored_ssh.join("config");
    assert!(restored_conf.exists());
    assert_eq!(
        fs::metadata(&restored_conf).unwrap().permissions().mode() & 0o777,
        0o600
    );

    // Verify restored custom directories
    let restored_custom_file = restore_target_custom.join("my_docs/project.md");
    assert!(restored_custom_file.exists());
    assert_eq!(
        fs::read_to_string(restored_custom_file).unwrap(),
        "# Secret Distro Project"
    );

    // 7. Test Selective Restore with SelectionFilter
    let mut selective_filter = SelectionFilter::new();
    selective_filter.set_restore_zen(false);
    selective_filter.set_restore_git_config(false);
    selective_filter.exclude_ssh_key("id_ed25519".to_string());
    selective_filter.toggle_custom_dir("my_docs".to_string());

    let selective_home = test_root.join("selective_home");
    let selective_config = test_root.join("selective_config");
    let selective_custom = test_root.join("selective_custom");

    let sel_req = RestoreRequest::new(&output_archive, &selective_config, &selective_home)
        .with_custom_base(&selective_custom)
        .with_filter(selective_filter);

    let sel_report = restore_use_case
        .execute(sel_req)
        .expect("Selective restore failed");
    assert_eq!(sel_report.profiles_restored.len(), 0);
    assert_eq!(sel_report.git_configs_restored, 0);
    assert_eq!(sel_report.ssh_keys_restored, 2);
    assert!(!selective_home.join(".ssh/id_ed25519").exists());
    assert!(selective_home.join(".ssh/id_ed25519.pub").exists());
    assert!(!selective_custom.join("my_docs/project.md").exists());
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
        Arc::new(LinuxSshAdapter::new()),
        Arc::new(LinuxGitConfigAdapter::new()),
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
        Arc::new(LinuxSshAdapter::new()),
        Arc::new(LinuxGitConfigAdapter::new()),
        vec![],
    );

    let output_archive = tmp.path().join("filtered.tar.zst");
    let mut filter = SelectionFilter::new();
    filter.toggle_skill("unwanted-skill".to_string());
    filter.toggle_package(PackageName::new("ripgrep").unwrap());

    let req = BackupRequest::new(&zen_config, &user_home, &output_archive).with_filter(filter);
    let report = backup_use_case.execute(req).expect("Backup failed");

    // "unwanted-skill" should NOT be in manifest
    assert!(!report
        .manifest
        .agent_governance
        .skills
        .iter()
        .any(|s| s.name == "unwanted-skill"));
    assert!(report
        .manifest
        .agent_governance
        .skills
        .iter()
        .any(|s| s.name == "clean-code"));
}

#[test]
fn test_transcribe_path_to_target_home() {
    use application::transcribe_path_to_target_home;
    use std::path::{Path, PathBuf};

    let target_home = Path::new("/home/newuser");

    // Re-rooting /home/<olduser>/...
    let original = Path::new("/home/olduser/Documents/eduardorwll");
    let transcribed = transcribe_path_to_target_home(original, target_home);
    assert_eq!(
        transcribed,
        PathBuf::from("/home/newuser/Documents/eduardorwll")
    );

    let original_ssh = Path::new("/home/archuser/.ssh");
    let transcribed_ssh = transcribe_path_to_target_home(original_ssh, target_home);
    assert_eq!(transcribed_ssh, PathBuf::from("/home/newuser/.ssh"));

    // Relative path
    let rel = Path::new("Projects/distro-vault");
    let transcribed_rel = transcribe_path_to_target_home(rel, target_home);
    assert_eq!(
        transcribed_rel,
        PathBuf::from("/home/newuser/Projects/distro-vault")
    );

    // System path outside /home
    let sys = Path::new("/opt/custom_app");
    let transcribed_sys = transcribe_path_to_target_home(sys, target_home);
    assert_eq!(transcribed_sys, PathBuf::from("/opt/custom_app"));
}

#[test]
fn test_slice_snapshot_filters_and_repackages_correctly() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let test_root = tmp.path();

    // 1. Setup mock source environment for initial backup
    let zen_config = test_root.join(".config/zen");
    let prof_dir = zen_config.join("7awtlvx8.Default (release)");
    fs::create_dir_all(&prof_dir).unwrap();

    let session_data = r#"{"spaces":[{"id":"s1","name":"DevSpace","tabs":["t1","t2"]}]}"#;
    let compressed_session = MozLz4Codec::compress(session_data).unwrap();
    fs::write(prof_dir.join("zen-sessions.jsonlz4"), compressed_session).unwrap();

    let user_home = test_root.join("home_user");
    let skills_dir = user_home.join(".agents/skills/clean-code");
    fs::create_dir_all(&skills_dir).unwrap();
    fs::write(
        skills_dir.join("SKILL.md"),
        "---\nname: clean-code\ndescription: Clean Code principles.\n---\nBody",
    )
    .unwrap();
    fs::write(user_home.join("GEMINI.md"), "# Guidelines\nDefault dir").unwrap();

    let ssh_dir = user_home.join(".ssh");
    fs::create_dir_all(&ssh_dir).unwrap();
    fs::write(ssh_dir.join("id_ed25519"), "MOCK_KEY_DATA").unwrap();
    fs::write(ssh_dir.join("id_ed25519.pub"), "ssh-ed25519 MOCK_PUB").unwrap();

    fs::write(
        user_home.join(".gitconfig"),
        "[user]\n\tname = Dev User\n\temail = dev@example.com\n",
    )
    .unwrap();

    let storage = Arc::new(ZstdStorageAdapter::new());
    let backup_use_case = BackupPipelineUseCase::new(
        Arc::new(ZenBrowserAdapter::new()),
        storage.clone(),
        Arc::new(LocalAgentAdapter::new()),
        Arc::new(LinuxSystemAdapter::new()),
        Arc::new(LinuxSshAdapter::new()),
        Arc::new(LinuxGitConfigAdapter::new()),
        vec![],
    );

    let full_archive = test_root.join("full_snapshot.tar.zst");
    let backup_req = BackupRequest::new(zen_config, user_home.clone(), full_archive.clone());
    let backup_rep = backup_use_case.execute(backup_req).unwrap();
    assert_eq!(backup_rep.manifest.ssh_keys.len(), 2);
    assert_eq!(backup_rep.manifest.git_configs.len(), 1);

    // 2. Setup slice request: exclude git config, exclude private key
    let mut filter = SelectionFilter::default();
    filter.set_restore_git_config(false);
    filter.toggle_ssh_key("id_ed25519".to_string()); // Exclude private key

    let sliced_archive = test_root.join("sliced_snapshot.tar.zst");
    let slice_use_case = SliceSnapshotUseCase::new(storage.clone());
    let slice_req = SliceSnapshotRequest::new(&full_archive, &sliced_archive, filter);

    let slice_rep = slice_use_case.execute(slice_req).unwrap();
    assert!(sliced_archive.exists());
    assert_eq!(slice_rep.output_path, sliced_archive);

    // Verify manifest of the sliced archive
    let inspected_manifest = storage.read_manifest(&sliced_archive).unwrap();
    assert!(inspected_manifest.git_configs.is_empty());
    assert_eq!(inspected_manifest.ssh_keys.len(), 1);
    assert_eq!(inspected_manifest.ssh_keys[0].file_name, "id_ed25519.pub");
    assert_eq!(inspected_manifest.zen_profiles.len(), 1);
    assert_eq!(inspected_manifest.agent_governance.skills.len(), 1);

    // 3. Test defensive check: overwriting source onto itself is rejected
    let invalid_req =
        SliceSnapshotRequest::new(&full_archive, &full_archive, SelectionFilter::default());
    let err = slice_use_case.execute(invalid_req).unwrap_err();
    match err {
        DomainError::ManifestError { message } => {
            assert!(message.contains("cannot be identical"));
        }
        other => panic!("Expected ManifestError, got {:?}", other),
    }
}
