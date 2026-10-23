use std::fs;
use std::path::Path;
use tempfile::tempdir;

use adapter_browser::{MozLz4Codec, ProfilesIniNormalizer, ZenBrowserAdapter};
use domain::errors::DomainError;
use domain::ports::BrowserPort;

#[test]
fn test_mozlz4_roundtrip_valid_payload() {
    let sample_json = r#"{"windows":[{"tabs":[{"entries":[{"url":"https://rust-lang.org","title":"Rust"}]}]}]}"#;
    let compressed = MozLz4Codec::compress(sample_json).expect("Compression failed");

    assert!(compressed.starts_with(b"mozLz40\0"));
    assert!(compressed.len() > 12);

    let decompressed = MozLz4Codec::decompress(&compressed).expect("Decompression failed");
    assert_eq!(decompressed, sample_json);
}

#[test]
fn test_mozlz4_rejects_truncated_payload() {
    let too_short = b"mozLz40\0\x01\x00";
    let err = MozLz4Codec::decompress(too_short).unwrap_err();
    match err {
        DomainError::BrowserError { message } => {
            assert!(message.contains("truncated"));
        }
        _ => panic!("Expected BrowserError for truncated payload"),
    }
}

#[test]
fn test_mozlz4_rejects_invalid_magic_header() {
    let mut payload = Vec::new();
    payload.extend_from_slice(b"badmagic");
    payload.extend_from_slice(&10u32.to_le_bytes());
    payload.extend_from_slice(b"0123456789");

    let err = MozLz4Codec::decompress(&payload).unwrap_err();
    match err {
        DomainError::BrowserError { message } => {
            assert!(message.contains("Invalid MozLZ4 magic header"));
        }
        _ => panic!("Expected BrowserError for invalid magic header"),
    }
}

#[test]
fn test_mozlz4_decompression_bomb_guard() {
    let mut bomb_payload = Vec::new();
    bomb_payload.extend_from_slice(b"mozLz40\0");
    // Declare 200 MB uncompressed size (limit is 128 MB)
    let declared_size = 200 * 1024 * 1024u32;
    bomb_payload.extend_from_slice(&declared_size.to_le_bytes());
    bomb_payload.extend_from_slice(b"fake compressed bytes");

    let err = MozLz4Codec::decompress(&bomb_payload).unwrap_err();
    match err {
        DomainError::DecompressionBombDetected { size } => {
            assert_eq!(size, declared_size as usize);
        }
        _ => panic!("Expected DecompressionBombDetected error"),
    }
}

#[test]
fn test_profiles_ini_normalizer_rewriting() {
    let sample_profiles_ini = r#"[General]
StartWithLastProfile=0
Version=2

[Profile0]
Name=Default (release)
IsRelative=1
Path=7awtlvx8.Default (release)

[Install15B76BAA26BA15E7]
Default=vo0xpeic.Default Profile
Locked=1

[Profile1]
Name=Default Profile
IsRelative=1
Path=vo0xpeic.Default Profile
Default=1
"#;

    let target = "7awtlvx8.Default (release)";
    let rewritten = ProfilesIniNormalizer::rewrite_profiles_ini(sample_profiles_ini, target);

    assert!(rewritten.contains("StartWithLastProfile=1"));
    assert!(rewritten.contains(&format!("Default={}", target)));
    assert!(rewritten.contains(&format!("Path={}\nDefault=1", target)) || rewritten.contains("Default=1"));

    let sample_installs_ini = r#"[15B76BAA26BA15E7]
Default=vo0xpeic.Default Profile
Locked=1
"#;

    let rewritten_installs = ProfilesIniNormalizer::rewrite_installs_ini(sample_installs_ini, target);
    assert!(rewritten_installs.contains(&format!("Default={}", target)));
    assert!(rewritten_installs.contains("Locked=1"));
}

#[test]
fn test_profiles_ini_normalizer_creates_files_if_missing() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let target = "my-restored-profile";

    ProfilesIniNormalizer::normalize(tmp.path(), target).expect("Normalization failed");

    let prof_ini = tmp.path().join("profiles.ini");
    let inst_ini = tmp.path().join("installs.ini");

    assert!(prof_ini.exists());
    assert!(inst_ini.exists());

    let prof_text = fs::read_to_string(&prof_ini).expect("Failed to read profiles.ini");
    let inst_text = fs::read_to_string(&inst_ini).expect("Failed to read installs.ini");

    assert!(prof_text.contains("StartWithLastProfile=1"));
    assert!(prof_text.contains(&format!("Path={}", target)));
    assert!(inst_text.contains(&format!("Default={}", target)));
}

#[test]
fn test_browser_adapter_scan_and_lock_check() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let adapter = ZenBrowserAdapter::new();

    let prof_dir = tmp.path().join("test.profile");
    fs::create_dir_all(&prof_dir).expect("Failed to create profile dir");

    // Initially not locked
    assert!(!adapter.is_locked(&prof_dir).expect("Failed to check lock"));

    // Add .parentlock
    let lock_file = prof_dir.join(".parentlock");
    fs::write(&lock_file, b"").expect("Failed to create lock file");
    assert!(adapter.is_locked(&prof_dir).expect("Failed to check lock"));

    // Add fake session
    let session_file = prof_dir.join("zen-sessions.jsonlz4");
    let compressed_session = MozLz4Codec::compress(r#"{"spaces":[{"id":"s1","name":"Dev","tabs":["t1","t2"]}]}"#).unwrap();
    fs::write(&session_file, compressed_session).unwrap();

    let scanned = adapter.scan_profiles(tmp.path()).expect("Scan failed");
    assert_eq!(scanned.len(), 1);
    assert_eq!(scanned[0].id.as_str(), "test.profile");
    assert_eq!(scanned[0].spaces.len(), 1);
    assert_eq!(scanned[0].spaces[0].name, "Dev");
    assert_eq!(scanned[0].spaces[0].open_tabs_count, 2);
}

#[test]
fn test_real_system_zen_sessions_lz4_if_present() {
    let real_session_path = Path::new("/home/eduardorwll/.config/zen/7awtlvx8.Default (release)/zen-sessions.jsonlz4");
    if real_session_path.exists() {
        let bytes = fs::read(real_session_path).expect("Failed to read real session");
        let decompressed = MozLz4Codec::decompress(&bytes).expect("Failed to decompress real zen-sessions.jsonlz4");
        assert!(decompressed.len() > 100);
        assert!(decompressed.contains("windows") || decompressed.contains("spaces") || decompressed.contains("tabs"));
    }
}
