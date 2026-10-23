use adapter_cli::args::{Cli, Commands};
use adapter_cli::tui::{TuiApp, TuiItem};
use clap::Parser;
use domain::value_objects::PackageName;
use tempfile::tempdir;

#[test]
fn test_cli_parsing_subcommands() {
    // 1. Backup with exclusions and custom output
    let parsed_backup = Cli::try_parse_from([
        "distro-vault",
        "backup",
        "--output",
        "/tmp/snapshot.tar.zst",
        "--exclude-package",
        "ripgrep",
        "--exclude-skill",
        "clean-code",
        "--dry-run",
    ])
    .expect("Failed to parse backup args");

    match parsed_backup.command {
        Some(Commands::Backup(b)) => {
            assert_eq!(b.output.unwrap().to_str().unwrap(), "/tmp/snapshot.tar.zst");
            assert_eq!(b.exclude_packages, vec!["ripgrep"]);
            assert_eq!(b.exclude_skills, vec!["clean-code"]);
            assert!(b.dry_run);
        }
        _ => panic!("Expected Backup command"),
    }

    // 2. Restore command
    let parsed_restore = Cli::try_parse_from([
        "distro-vault",
        "restore",
        "/tmp/snapshot.tar.zst",
        "--install-packages",
        "--dry-run",
    ])
    .expect("Failed to parse restore args");

    match parsed_restore.command {
        Some(Commands::Restore(r)) => {
            assert_eq!(r.archive.to_str().unwrap(), "/tmp/snapshot.tar.zst");
            assert!(r.install_packages);
            assert!(r.dry_run);
        }
        _ => panic!("Expected Restore command"),
    }

    // 3. Inspect command
    let parsed_inspect = Cli::try_parse_from([
        "distro-vault",
        "inspect",
        "/tmp/snapshot.tar.zst",
        "--json",
    ])
    .expect("Failed to parse inspect args");

    match parsed_inspect.command {
        Some(Commands::Inspect(i)) => {
            assert_eq!(i.archive.to_str().unwrap(), "/tmp/snapshot.tar.zst");
            assert!(i.json);
        }
        _ => panic!("Expected Inspect command"),
    }

    // 4. Verify command
    let parsed_verify = Cli::try_parse_from([
        "distro-vault",
        "verify",
        "/tmp/snapshot.tar.zst",
    ])
    .expect("Failed to parse verify args");

    match parsed_verify.command {
        Some(Commands::Verify(v)) => {
            assert_eq!(v.archive.to_str().unwrap(), "/tmp/snapshot.tar.zst");
        }
        _ => panic!("Expected Verify command"),
    }
}

#[test]
fn test_tui_app_state_and_filter_generation() {
    let mut app = TuiApp::new();

    // Setup dummy user packages
    app.user_packages.push(TuiItem {
        id: "ripgrep".to_string(),
        name: "ripgrep".to_string(),
        details: "[CliTool] • fast grep".to_string(),
        included: true,
    });
    app.user_packages.push(TuiItem {
        id: "git".to_string(),
        name: "git".to_string(),
        details: "[Development] • vcs".to_string(),
        included: true,
    });

    // Setup dummy system packages
    app.system_packages.push(TuiItem {
        id: "linux".to_string(),
        name: "linux".to_string(),
        details: "[KernelFirmware] • core kernel".to_string(),
        included: true,
    });

    // Setup dummy skills
    app.skills.push(TuiItem {
        id: "clean-code".to_string(),
        name: "clean-code".to_string(),
        details: "Uncle Bob".to_string(),
        included: true,
    });

    // 1. Navigation across tabs
    assert_eq!(app.current_tab, 0);
    app.next_tab();
    assert_eq!(app.current_tab, 1);
    app.prev_tab();
    assert_eq!(app.current_tab, 0);

    // 2. Cursor navigation
    assert_eq!(app.cursors[0], 0);
    app.next_item();
    assert_eq!(app.cursors[0], 1);

    // 3. Toggle selection (exclude git)
    app.toggle_selected();
    assert!(!app.user_packages[1].included);

    // 4. Build filter
    let filter = app.build_selection_filter();
    assert!(filter.is_package_included(&PackageName::new("ripgrep").unwrap()));
    assert!(!filter.is_package_included(&PackageName::new("git").unwrap()));
    assert!(filter.is_package_included(&PackageName::new("linux").unwrap()));
    assert!(filter.is_skill_included("clean-code"));

    // 5. Custom Directory Addition
    let tmp = tempdir().expect("Failed to create tempdir");
    let custom_p = tmp.path().join("my_custom_folder");
    std::fs::create_dir_all(&custom_p).unwrap();
    std::fs::write(custom_p.join("data.txt"), b"sample").unwrap();

    let res = app.add_custom_directory(custom_p.to_str().unwrap());
    assert!(res.is_ok());
    assert_eq!(app.custom_dirs.len(), 1);
    assert_eq!(app.custom_dirs[0].file_count, 1);
}
