use std::path::PathBuf;

use domain::{
    AgentGovernance, AppImageEntry, Checksum, Container, DistroFamily, Manifest, PackageEntry,
    PackageName, PackageScope, PackageSource, ProfileId, RuleEntry, SkillCategory, SkillEntry,
    Space, UnixMode, ZenProfile,
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
    assert_eq!(DistroFamily::from_id("ubuntu", Some("debian")), DistroFamily::Debian);
    assert_eq!(DistroFamily::from_id("opensuse-tumbleweed", None), DistroFamily::OpenSuse);
    assert_eq!(DistroFamily::from_id("unknown_os", None), DistroFamily::Unknown);
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
    let mut profile = ZenProfile::new(pid, true, PathBuf::from("/home/user/.config/zen/test.default"));

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

    let json = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    let deserialized: Manifest = serde_json::from_str(&json).expect("deserialize manifest");

    assert_eq!(manifest.snapshot_id, deserialized.snapshot_id);
    assert_eq!(manifest.source_distro, deserialized.source_distro);
    assert_eq!(deserialized.packages.len(), 1);
    assert_eq!(deserialized.appimages.len(), 1);
}
