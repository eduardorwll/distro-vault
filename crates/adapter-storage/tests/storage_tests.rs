use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::{Builder, Header};
use tempfile::tempdir;

use adapter_storage::{CustomDirsHandler, ZipSlipGuard, ZstdStorageAdapter};
use domain::entities::agent_governance::AgentGovernance;
use domain::entities::Manifest;
use domain::errors::DomainError;
use domain::ports::StoragePort;
use domain::value_objects::DistroFamily;

#[test]
fn test_storage_roundtrip_snapshot_and_manifest() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let source_dir = tmp.path().join("source");
    fs::create_dir_all(&source_dir).unwrap();

    let file_a = source_dir.join("hello.txt");
    fs::write(&file_a, b"Hello Distro-Vault!").unwrap();

    let custom_sub = source_dir.join("my_studies");
    fs::create_dir_all(&custom_sub).unwrap();
    fs::write(custom_sub.join("notes.md"), b"# Rust & Clean Architecture").unwrap();

    let custom_entry = CustomDirsHandler::inspect_directory(&custom_sub, Some("estudos")).unwrap();
    assert_eq!(custom_entry.label, "estudos");
    assert_eq!(custom_entry.file_count, 1);
    assert!(custom_entry.size_bytes > 0);

    let mut manifest = Manifest::new(
        "snap-001",
        "2026-10-23T12:00:00Z",
        "arch-station",
        DistroFamily::Arch,
        AgentGovernance::new("~/Antigravity"),
    );
    manifest.custom_directories.push(custom_entry.clone());

    let archive_path = tmp.path().join("backup.tar.zst");
    let adapter = ZstdStorageAdapter::new();

    let items = vec![
        (file_a.clone(), PathBuf::from("root_files/hello.txt")),
        CustomDirsHandler::to_archive_item(&custom_entry),
    ];

    let checksum = adapter
        .create_snapshot(&items, &manifest, &archive_path)
        .expect("Create snapshot failed");
    assert_eq!(checksum.as_str().len(), 64);
    assert!(archive_path.exists());

    // Read manifest without full unpacking
    let read_m = adapter
        .read_manifest(&archive_path)
        .expect("Read manifest failed");
    assert_eq!(read_m.snapshot_id, "snap-001");
    assert_eq!(read_m.source_host, "arch-station");
    assert_eq!(read_m.source_distro, DistroFamily::Arch);
    assert_eq!(read_m.custom_directories.len(), 1);

    // Unpack to target
    let restore_dir = tmp.path().join("restore");
    adapter
        .extract_snapshot(&archive_path, &restore_dir)
        .expect("Extraction failed");

    assert!(restore_dir.join("manifest.json").exists());
    assert!(restore_dir.join("root_files/hello.txt").exists());
    assert!(restore_dir.join("custom/my_studies/notes.md").exists());
    assert_eq!(
        fs::read(restore_dir.join("custom/my_studies/notes.md")).unwrap(),
        b"# Rust & Clean Architecture"
    );
}

#[test]
fn test_zip_slip_rejection() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let target_dir = tmp.path().join("extract_target");
    fs::create_dir_all(&target_dir).unwrap();
    let canonical_target = target_dir.canonicalize().unwrap();

    // 1. Unit test ZipSlipGuard directly
    assert!(
        ZipSlipGuard::sanitize_entry_path(Path::new("../escaped.txt"), &canonical_target).is_err()
    );
    assert!(
        ZipSlipGuard::sanitize_entry_path(Path::new("/absolute/file.txt"), &canonical_target)
            .is_err()
    );
    assert!(ZipSlipGuard::sanitize_entry_path(
        Path::new("safe/nested/file.txt"),
        &canonical_target
    )
    .is_ok());

    // 2. Integration test with raw tar header
    let malicious_archive_path = tmp.path().join("malicious.tar.zst");
    let out_file = File::create(&malicious_archive_path).unwrap();
    let zstd_enc = zstd::stream::write::Encoder::new(out_file, 3).unwrap();
    let mut tar_b = Builder::new(zstd_enc);

    let mut header = Header::new_gnu();
    let payload = b"MALICIOUS DATA";
    header.set_size(payload.len() as u64);
    header.set_mode(0o644);

    let name_bytes = b"../escaped.txt\0";
    header.as_mut_bytes()[..name_bytes.len()].copy_from_slice(name_bytes);
    header.set_cksum();

    tar_b.append(&header, &payload[..]).unwrap();
    let enc = tar_b.into_inner().unwrap();
    enc.finish().unwrap();

    let adapter = ZstdStorageAdapter::new();
    let err = adapter
        .extract_snapshot(&malicious_archive_path, &target_dir)
        .unwrap_err();

    match err {
        DomainError::SecurityViolation { path } => {
            assert!(path.contains("Zip-Slip") || path.contains("escaped.txt"));
        }
        _ => panic!("Expected SecurityViolation for zip-slip, got: {:?}", err),
    }

    assert!(!tmp.path().join("escaped.txt").exists());
}

#[test]
fn test_checksum_computation() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let file_path = tmp.path().join("test.bin");
    fs::write(&file_path, b"SHA256 Test Vector").unwrap();

    let adapter = ZstdStorageAdapter::new();
    let checksum = adapter.compute_checksum(&file_path).unwrap();
    assert_eq!(checksum.as_str().len(), 64);

    // Verify deterministic checksum
    let checksum_again = adapter.compute_checksum(&file_path).unwrap();
    assert_eq!(checksum, checksum_again);
}
