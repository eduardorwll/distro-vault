use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

use adapter_system::{
    AptAdapter, AurAdapter, DnfAdapter, FlatpakAdapter, LinuxSshAdapter, LinuxSystemAdapter,
    OsReleaseParser, PacmanAdapter,
};
use domain::entities::package::PackageSource;
use domain::entities::ssh_key::SshFileType;
use domain::ports::{PackagePort, SshPort, SystemPort};
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

#[test]
fn test_ssh_adapter_scan_and_permissions() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let fake_home = tmp.path().join("home_user");
    let fake_ssh = fake_home.join(".ssh");
    fs::create_dir_all(&fake_ssh).unwrap();

    fs::write(fake_ssh.join("id_ed25519"), "-----BEGIN OPENSSH PRIVATE KEY-----\nMOCK\n-----END OPENSSH PRIVATE KEY-----").unwrap();
    fs::write(fake_ssh.join("id_ed25519.pub"), "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5 mock@test").unwrap();
    fs::write(fake_ssh.join("config"), "Host *\n  ServerAliveInterval 60\n").unwrap();
    fs::write(fake_ssh.join("authorized_keys"), "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5 user@client\n").unwrap();
    fs::write(fake_ssh.join("known_hosts"), "github.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5\n").unwrap();
    fs::write(fake_ssh.join("id_ed25519-cert.pub"), "ssh-ed25519-cert-v01@openssh.com CERTIFICATE\n").unwrap();

    let adapter = LinuxSshAdapter::new();
    let keys = adapter.scan_ssh_keys(&fake_home).expect("Scan SSH keys failed");

    assert_eq!(keys.len(), 6);

    let find_type = |name: &str| keys.iter().find(|k| k.file_name == name).unwrap().file_type;
    assert_eq!(find_type("id_ed25519"), SshFileType::PrivateKey);
    assert_eq!(find_type("id_ed25519.pub"), SshFileType::PublicKey);
    assert_eq!(find_type("config"), SshFileType::Config);
    assert_eq!(find_type("authorized_keys"), SshFileType::AuthorizedKeys);
    assert_eq!(find_type("known_hosts"), SshFileType::KnownHosts);
    assert_eq!(find_type("id_ed25519-cert.pub"), SshFileType::Certificate);

    // Test restoration with POSIX permission hardening
    let target_home = tmp.path().join("restored_home");
    let restored_count = adapter.restore_ssh_keys(&fake_ssh, &target_home, &keys, false).expect("Restore SSH keys failed");
    assert_eq!(restored_count, 6);

    let restored_ssh = target_home.join(".ssh");
    assert!(restored_ssh.exists());
    assert_eq!(fs::metadata(&restored_ssh).unwrap().permissions().mode() & 0o777, 0o700);

    let mode_of = |file: &str| fs::metadata(restored_ssh.join(file)).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode_of("id_ed25519"), 0o600);
    assert_eq!(mode_of("id_ed25519.pub"), 0o644);
    assert_eq!(mode_of("config"), 0o600);
    assert_eq!(mode_of("authorized_keys"), 0o600);
    assert_eq!(mode_of("known_hosts"), 0o644);
    assert_eq!(mode_of("id_ed25519-cert.pub"), 0o644);
}
