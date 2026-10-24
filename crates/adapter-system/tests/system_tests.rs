use adapter_system::{AptAdapter, AurAdapter, DnfAdapter, FlatpakAdapter, LinuxSystemAdapter, OsReleaseParser, PacmanAdapter};
use domain::entities::package::PackageSource;
use domain::ports::{PackagePort, SystemPort};
use domain::value_objects::DistroFamily;

#[test]
fn test_os_release_parser_matrix() {
    let arch_sample = "NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\nID=arch\nBUILD_ID=rolling\nANSI_COLOR=\"38;2;23;147;209\"\n";
    assert_eq!(OsReleaseParser::parse_content(arch_sample).unwrap(), DistroFamily::Arch);

    let omarchy_sample = "NAME=\"Omarchy\"\nID=omarchy\nID_LIKE=arch\n";
    assert_eq!(OsReleaseParser::parse_content(omarchy_sample).unwrap(), DistroFamily::Arch);

    let fedora_sample = "NAME=Fedora\nVERSION=\"40 (Workstation Edition)\"\nID=fedora\nVERSION_ID=40\nPLATFORM_ID=\"platform:f40\"\n";
    assert_eq!(OsReleaseParser::parse_content(fedora_sample).unwrap(), DistroFamily::Fedora);

    let ubuntu_sample = "NAME=\"Ubuntu\"\nVERSION=\"24.04 LTS\"\nID=ubuntu\nID_LIKE=debian\n";
    assert_eq!(OsReleaseParser::parse_content(ubuntu_sample).unwrap(), DistroFamily::Debian);

    let invalid_sample = "FOO=bar\nBAZ=qux\n";
    assert!(OsReleaseParser::parse_content(invalid_sample).is_err());
}

#[test]
fn test_real_system_adapter() {
    let adapter = LinuxSystemAdapter::new();
    let distro = adapter.detect_distro().expect("Detect distro failed on real host");
    assert_eq!(distro, DistroFamily::Arch);

    let hostname = adapter.hostname().expect("Read hostname failed");
    assert!(!hostname.is_empty());

    assert!(!adapter.is_process_running("nonexistent_process_super_fake_12345"));
}

#[test]
fn test_real_system_package_query() {
    let pacman = PacmanAdapter::new();
    let native_pkgs = pacman.query_installed().expect("Query native packages failed");
    assert!(!native_pkgs.is_empty(), "Native packages list should not be empty on Arch");

    // Spot check presence of standard tools like coreutils, pacman or zen-browser-bin
    let has_native = native_pkgs.iter().any(|p| p.name.as_str().contains("zen") || p.name.as_str() == "pacman" || p.name.as_str() == "git");
    assert!(has_native, "Expected common package in pacman list");

    let aur = AurAdapter::new();
    if let Ok(aur_pkgs) = aur.query_installed() {
        let has_aur = aur_pkgs.iter().any(|p| p.name.as_str() == "paru" || p.name.as_str().contains("antigravity") || p.name.as_str().contains("rustrover"));
        assert!(has_aur, "Expected paru or antigravity in AUR packages");
    }

    let flatpak = FlatpakAdapter::new();
    if let Ok(flatpaks) = flatpak.query_installed() {
        // Gear Lever was seen in system
        let has_gear = flatpaks.iter().any(|p| p.name.as_str().contains("gearlever"));
        assert!(has_gear, "Expected gearlever in flatpaks");
    }
}

#[test]
fn test_apt_and_dnf_adapters_resilience() {
    let dnf = DnfAdapter::new();
    assert_eq!(dnf.source(), PackageSource::DnfNative);
    // On systems without dnf, query_installed returns Ok(empty vec) safely without panic or error
    let dnf_pkgs = dnf.query_installed().expect("DnfAdapter query_installed must not error");
    let dnf_deps = dnf.query_dependencies().expect("DnfAdapter query_dependencies must not error");
    // Verify results are valid collections
    let _ = dnf_pkgs.len();
    let _ = dnf_deps.len();

    let apt = AptAdapter::new();
    assert_eq!(apt.source(), PackageSource::AptNative);
    // On systems without apt, query_installed returns Ok(empty vec) safely without panic or error
    let apt_pkgs = apt.query_installed().expect("AptAdapter query_installed must not error");
    let apt_deps = apt.query_dependencies().expect("AptAdapter query_dependencies must not error");
    // Verify results are valid collections
    let _ = apt_pkgs.len();
    let _ = apt_deps.len();
}
