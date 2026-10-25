use std::path::PathBuf;

use domain::{
    AgentGovernance, AppImageEntry, Checksum, Container, CustomDirectoryEntry, DistroFamily,
    Manifest, PackageEntry, PackageName, PackageScope, PackageSource, ProfileId, RuleEntry,
    SelectionFilter, SkillCategory, SkillEntry, Space, UnixMode, ZenProfile,
};

#[test]
fn test_profile_id_validations() {
    assert!(ProfileId::new("").is_err());
    assert!(ProfileId::new("   ").is_err());

    let pid = ProfileId::new("7awtlvx8.Default (release)").expect("valid profile id");
    assert_eq!(pid.as_str(), "7awtlvx8.Default (release)");
}

#[test]
fn test_package_name_validations() {
    assert!(PackageName::new("").is_err());
    let pkg = PackageName::new("zen-browser-bin").expect("valid package name");
    assert_eq!(pkg.as_str(), "zen-browser-bin");
}

#[test]
fn test_checksum_validations() {
    assert!(Checksum::new("too_short").is_err());
    let valid_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let chk = Checksum::new(valid_sha256).expect("valid sha256");
    assert_eq!(chk.as_str(), valid_sha256);
}

#[test]
fn test_distro_family_inference() {
    assert_eq!(DistroFamily::from_id("omarchy", None), DistroFamily::Arch);
    assert_eq!(DistroFamily::from_id("arch", None), DistroFamily::Arch);
    assert_eq!(DistroFamily::from_id("cachyos", None), DistroFamily::Arch);
    assert_eq!(DistroFamily::from_id("fedora", None), DistroFamily::Fedora);
    assert_eq!(
        DistroFamily::from_id("ubuntu", Some("debian")),
        DistroFamily::Debian
    );
    assert_eq!(
        DistroFamily::from_id("opensuse-tumbleweed", None),
        DistroFamily::OpenSuse
    );
    assert_eq!(
        DistroFamily::from_id("unknown_os", None),
        DistroFamily::Unknown
    );
}

#[test]
fn test_unix_mode_permissions() {
    let mode_700 = UnixMode::SECURE_DIR;
    assert_eq!(mode_700.raw(), 0o700);
    assert!(mode_700.is_strictly_private());
    assert!(mode_700.is_executable());

    let mode_600 = UnixMode::SECURE_PRIVATE_KEY;
    assert_eq!(mode_600.raw(), 0o600);
    assert!(mode_600.is_strictly_private());
    assert!(!mode_600.is_executable());

    let mode_755 = UnixMode::EXECUTABLE_FILE;
    assert!(!mode_755.is_strictly_private());
    assert!(mode_755.is_executable());
}

#[test]
fn test_zen_profile_and_spaces() {
    let pid = ProfileId::new("test.default").unwrap();
    let mut profile = ZenProfile::new(
        pid,
        true,
        PathBuf::from("/home/user/.config/zen/test.default"),
    );

    profile.spaces.push(Space {
        id: "space-1".to_string(),
        name: "Desenvolvimento".to_string(),
        icon: Some("code".to_string()),
        color: Some("#ff0000".to_string()),
        container_id: Some("work".to_string()),
        open_tabs_count: 5,
        pinned_tabs_count: 2,
    });

    profile.spaces.push(Space {
        id: "space-2".to_string(),
        name: "Pessoal".to_string(),
        icon: None,
        color: None,
        container_id: None,
        open_tabs_count: 3,
        pinned_tabs_count: 0,
    });

    profile.containers.push(Container {
        id: "work".to_string(),
        name: "Trabalho".to_string(),
        icon: "briefcase".to_string(),
        color: "blue".to_string(),
    });

    assert_eq!(profile.spaces.len(), 2);
    assert_eq!(profile.total_tabs(), 10);
}

#[test]
fn test_agent_governance_and_skills() {
    let mut gov = AgentGovernance::new("~/Antigravity/");
    assert!(gov.quality_gates.mandatory_pre_approval);
    assert!(gov.quality_gates.five_phase_engineering_cycle);

    gov.rules.push(RuleEntry {
        name: "AGENTS.md".to_string(),
        relative_path: PathBuf::from("AGENTS.md"),
        description: "Regras de engenharia".to_string(),
        is_mandatory: true,
    });

    gov.skills.push(SkillEntry {
        name: "clean-architecture".to_string(),
        category: SkillCategory::Architecture,
        is_symlink: false,
        dereferenced: true,
        original_target: None,
        relative_path: PathBuf::from(".agents/skills/clean-architecture"),
        description: Some("Clean Architecture".to_string()),
    });

    gov.skills.push(SkillEntry {
        name: "security-and-hardening".to_string(),
        category: SkillCategory::Security,
        is_symlink: false,
        dereferenced: true,
        original_target: None,
        relative_path: PathBuf::from(".agents/skills/security-and-hardening"),
        description: Some("Security".to_string()),
    });

    let arch_skills = gov.skills_by_category(SkillCategory::Architecture);
    assert_eq!(arch_skills.len(), 1);
    assert_eq!(arch_skills[0].name, "clean-architecture");
}

#[test]
fn test_selection_filter_exclusions() {
    let mut filter = SelectionFilter::new();
    let pkg1 = PackageName::new("ripgrep").unwrap();
    let pkg2 = PackageName::new("bloat-package").unwrap();

    assert!(filter.is_package_included(&pkg1));
    assert!(filter.is_package_included(&pkg2));

    filter.toggle_package(pkg2.clone());
    assert!(filter.is_package_included(&pkg1));
    assert!(!filter.is_package_included(&pkg2));

    // Toggle back
    filter.toggle_package(pkg2.clone());
    assert!(filter.is_package_included(&pkg2));

    // Test skills exclusion
    assert!(filter.is_skill_included("clean-architecture"));
    filter.toggle_skill("clean-architecture".to_string());
    assert!(!filter.is_skill_included("clean-architecture"));
}

#[test]
fn test_custom_directory_entry() {
    let entry = CustomDirectoryEntry::new(
        PathBuf::from("/home/user/Documents/secret-notes"),
        PathBuf::from("custom/secret-notes"),
        "Secret Notes",
    )
    .with_stats(1024 * 50, 12);

    assert_eq!(entry.label, "Secret Notes");
    assert_eq!(entry.size_bytes, 1024 * 50);
    assert_eq!(entry.file_count, 12);
}

#[test]
fn test_manifest_roundtrip_serialization() {
    let gov = AgentGovernance::new("~/Antigravity/");
    let mut manifest = Manifest::new(
        "snap-2026-10-23",
        "2026-10-23T17:20:00Z",
        "omarchy-laptop",
        DistroFamily::Arch,
        gov,
    );

    manifest.packages.push(PackageEntry::new(
        PackageName::new("ripgrep").unwrap(),
        PackageSource::PacmanNative,
        PackageScope::System,
    ));

    manifest.appimages.push(AppImageEntry::new(
        "joplin.appimage",
        PathBuf::from("AppImages/joplin.appimage"),
        1024 * 1024,
        UnixMode::EXECUTABLE_FILE,
    ));

    manifest.custom_directories.push(CustomDirectoryEntry::new(
        PathBuf::from("/home/user/custom-dir"),
        PathBuf::from("custom/custom-dir"),
        "Meus Estudos",
    ));

    let json = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    let deserialized: Manifest = serde_json::from_str(&json).expect("deserialize manifest");

    assert_eq!(manifest.snapshot_id, deserialized.snapshot_id);
    assert_eq!(manifest.source_distro, deserialized.source_distro);
    assert_eq!(deserialized.packages.len(), 1);
    assert_eq!(deserialized.appimages.len(), 1);
    assert_eq!(deserialized.custom_directories.len(), 1);
    assert_eq!(deserialized.custom_directories[0].label, "Meus Estudos");
}

#[test]
fn test_package_category_classification() {
    use domain::entities::package::{PackageCategory, PackageScope, PackageSource};

    // Low-level system components
    let (s, c) = PackageCategory::classify("linux", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::KernelFirmware);

    let (s, c) = PackageCategory::classify("base-devel", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::BaseBoot);

    let (s, c) = PackageCategory::classify("btrfs-progs", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::FileSystems);

    let (s, c) = PackageCategory::classify("vulkan-radeon", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::DriversHardware);

    let (s, c) = PackageCategory::classify("networkmanager", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::NetworkDaemons);

    // Distro Pre-installed base packages (Non-user, System scope) - Arch, Fedora, Debian, etc.
    let (s, c) = PackageCategory::classify("ripgrep", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::DistroBase);

    let (s, c) = PackageCategory::classify("docker", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::DistroBase);

    let (s, c) = PackageCategory::classify("dnf", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::DistroBase);

    let (s, c) = PackageCategory::classify("apt", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::DistroBase);

    let (s, c) = PackageCategory::classify("omacalc", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::System);
    assert_eq!(c, PackageCategory::DistroBase);

    // User-installed components (not pre-installed by distro)
    let (s, c) = PackageCategory::classify("discord", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::User);
    assert_eq!(c, PackageCategory::DesktopApp);

    let (s, c) = PackageCategory::classify("dbeaver", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::User);
    assert_eq!(c, PackageCategory::DesktopApp);

    let (s, c) = PackageCategory::classify("antigravity-ide", PackageSource::PacmanNative);
    assert_eq!(s, PackageScope::User);
    assert_eq!(c, PackageCategory::DesktopApp);

    let (s, c) = PackageCategory::classify("org.mozilla.firefox", PackageSource::Flatpak);
    assert_eq!(s, PackageScope::User);
    assert_eq!(c, PackageCategory::DesktopApp);
}

#[test]
fn test_ssh_key_entity_and_filter() {
    use domain::entities::ssh_key::{SshFileType, SshKeyEntry};
    use std::path::PathBuf;

    let key = SshKeyEntry::new(
        "id_ed25519",
        PathBuf::from("id_ed25519"),
        SshFileType::PrivateKey,
        411,
        0o600,
    );
    assert_eq!(key.file_type.as_str(), "Chave Privada");
    assert_eq!(key.file_type.default_posix_mode(), 0o600);
    assert_eq!(SshFileType::PublicKey.default_posix_mode(), 0o644);
    assert_eq!(SshFileType::Config.default_posix_mode(), 0o600);

    let mut filter = domain::entities::SelectionFilter::new();
    assert!(filter.is_ssh_key_included("id_ed25519"));

    filter.exclude_ssh_key("id_ed25519".to_string());
    assert!(!filter.is_ssh_key_included("id_ed25519"));

    filter.include_ssh_key("id_ed25519");
    assert!(filter.is_ssh_key_included("id_ed25519"));

    filter.toggle_ssh_key("id_ed25519".to_string());
    assert!(!filter.is_ssh_key_included("id_ed25519"));
}

#[test]
fn test_git_config_entry_and_manifest_serialization() {
    use domain::entities::git_config::GitConfigEntry;

    let git_cfg = GitConfigEntry::new(
        ".gitconfig",
        ".gitconfig",
        Some("Eduardo".to_string()),
        Some("eduardo@example.com".to_string()),
        120,
    );

    assert_eq!(git_cfg.relative_path, ".gitconfig");
    assert_eq!(git_cfg.file_name, ".gitconfig");
    assert_eq!(git_cfg.user_name.as_deref(), Some("Eduardo"));
    assert_eq!(git_cfg.user_email.as_deref(), Some("eduardo@example.com"));

    let json = serde_json::to_string(&git_cfg).expect("serialize git config");
    let deserialized: GitConfigEntry = serde_json::from_str(&json).expect("deserialize git config");
    assert_eq!(git_cfg, deserialized);
}

#[test]
fn test_selection_filter_granular_restore_options() {
    let mut filter = domain::entities::SelectionFilter::new();
    assert!(filter.restore_zen);
    assert!(filter.restore_git_config);
    assert!(filter.is_custom_dir_included("my-docs"));

    filter.set_restore_zen(false);
    assert!(!filter.restore_zen);

    filter.set_restore_git_config(false);
    assert!(!filter.restore_git_config);

    filter.toggle_custom_dir("my-docs".to_string());
    assert!(!filter.is_custom_dir_included("my-docs"));

    filter.toggle_custom_dir("my-docs".to_string());
    assert!(filter.is_custom_dir_included("my-docs"));
}

#[test]
fn test_vault_config_defaults_and_manipulation() {
    use domain::entities::VaultConfig;
    use std::path::{Path, PathBuf};

    let home = Path::new("/home/testuser");
    let mut config = VaultConfig::default_for_home(home);

    assert_eq!(config.output_dir, PathBuf::from("/home/testuser/Antigravity/backups"));
    assert_eq!(config.input_dirs.len(), 2);
    assert!(config.auto_install_packages);
    assert!(!config.compact_view);

    // Test add and remove input dirs
    let added = config.add_input_dir("/mnt/external/backups");
    assert!(added);
    assert_eq!(config.input_dirs.len(), 3);

    // Duplicate add should return false
    let added_dup = config.add_input_dir("/mnt/external/backups");
    assert!(!added_dup);
    assert_eq!(config.input_dirs.len(), 3);

    // Set new output dir
    config.set_output_dir("/mnt/external/new_backups");
    assert_eq!(config.output_dir, PathBuf::from("/mnt/external/new_backups"));
    assert!(config.input_dirs.contains(&PathBuf::from("/mnt/external/new_backups")));

    // Remove input dir
    let removed = config.remove_input_dir(Path::new("/home/testuser/Antigravity/backups"));
    assert!(removed);
    assert!(!config.input_dirs.contains(&PathBuf::from("/home/testuser/Antigravity/backups")));

    // Toggles
    config.toggle_compact_view();
    assert!(config.compact_view);
    config.toggle_auto_install_packages();
    assert!(!config.auto_install_packages);
}
