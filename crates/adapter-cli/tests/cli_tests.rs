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

    // 2. Restore command (with explicit archive)
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
            assert_eq!(
                r.archive.as_ref().unwrap().to_str().unwrap(),
                "/tmp/snapshot.tar.zst"
            );
            assert!(r.install_packages);
            assert!(r.dry_run);
        }
        _ => panic!("Expected Restore command"),
    }

    // 2.1 Restore command (interactive, without archive)
    let parsed_interactive_restore = Cli::try_parse_from(["distro-vault", "restore"])
        .expect("Failed to parse interactive restore args");

    match parsed_interactive_restore.command {
        Some(Commands::Restore(r)) => {
            assert!(r.archive.is_none());
        }
        _ => panic!("Expected Restore command"),
    }

    // 3. Inspect command
    let parsed_inspect =
        Cli::try_parse_from(["distro-vault", "inspect", "/tmp/snapshot.tar.zst", "--json"])
            .expect("Failed to parse inspect args");

    match parsed_inspect.command {
        Some(Commands::Inspect(i)) => {
            assert_eq!(i.archive.to_str().unwrap(), "/tmp/snapshot.tar.zst");
            assert!(i.json);
        }
        _ => panic!("Expected Inspect command"),
    }

    // 4. Verify command
    let parsed_verify = Cli::try_parse_from(["distro-vault", "verify", "/tmp/snapshot.tar.zst"])
        .expect("Failed to parse verify args");

    match parsed_verify.command {
        Some(Commands::Verify(v)) => {
            assert_eq!(v.archive.to_str().unwrap(), "/tmp/snapshot.tar.zst");
        }
        _ => panic!("Expected Verify command"),
    }

    // 5. Slice command
    let parsed_slice = Cli::try_parse_from([
        "distro-vault",
        "slice",
        "/tmp/snapshot.tar.zst",
        "--output",
        "/tmp/subset.tar.zst",
        "--no-ssh",
        "--no-zen",
        "--no-git-config",
        "--exclude-package",
        "vim",
    ])
    .expect("Failed to parse slice args");

    match parsed_slice.command {
        Some(Commands::Slice(s)) => {
            assert_eq!(s.source.to_str().unwrap(), "/tmp/snapshot.tar.zst");
            assert_eq!(s.output.unwrap().to_str().unwrap(), "/tmp/subset.tar.zst");
            assert!(s.no_ssh);
            assert!(s.no_zen);
            assert!(s.no_git_config);
            assert_eq!(s.exclude_packages, vec!["vim"]);
        }
        _ => panic!("Expected Slice command"),
    }
}

#[test]
fn test_tui_app_state_and_filter_generation() {
    let mut app = TuiApp::new();

    // Setup dummy user packages with headers
    app.user_packages = TuiApp::group_into_rows(vec![(
        "cli".to_string(),
        "FERRAMENTAS CLI".to_string(),
        vec![
            TuiItem {
                id: "ripgrep".to_string(),
                name: "ripgrep".to_string(),
                details: "fast grep".to_string(),
                included: true,
            },
            TuiItem {
                id: "git".to_string(),
                name: "git".to_string(),
                details: "vcs".to_string(),
                included: true,
            },
        ],
    )]);

    // Setup dummy system packages with headers (unselected by default)
    app.system_packages = TuiApp::group_into_rows(vec![(
        "kernel".to_string(),
        "KERNEL & FIRMWARE".to_string(),
        vec![TuiItem {
            id: "linux".to_string(),
            name: "linux".to_string(),
            details: "core kernel".to_string(),
            included: false,
        }],
    )]);

    // Setup dummy dependency packages with headers (unselected by default)
    app.dependency_packages = TuiApp::group_into_rows(vec![(
        "core_libs".to_string(),
        "BIBLIOTECAS CORE".to_string(),
        vec![TuiItem {
            id: "glibc".to_string(),
            name: "glibc".to_string(),
            details: "C runtime".to_string(),
            included: false,
        }],
    )]);

    // Setup dummy skills with headers
    app.skills = TuiApp::group_into_rows(vec![(
        "clean_code".to_string(),
        "CLEAN CODE".to_string(),
        vec![TuiItem {
            id: "clean-code".to_string(),
            name: "clean-code".to_string(),
            details: "Uncle Bob".to_string(),
            included: true,
        }],
    )]);

    // Verify initial selection counts
    assert_eq!(app.count_items(0), 2);
    assert_eq!(app.count_selected_items(0), 2);
    assert_eq!(app.count_items(1), 1);
    assert_eq!(app.count_selected_items(1), 0); // System packages unselected by default
    assert_eq!(app.count_items(2), 1);
    assert_eq!(app.count_selected_items(2), 0); // Dependencies unselected by default

    // 1. Navigation across tabs (7 tabs now)
    assert_eq!(app.current_tab, 0);
    app.next_tab();
    assert_eq!(app.current_tab, 1);
    app.prev_tab();
    assert_eq!(app.current_tab, 0);

    // 2. Cursor navigation (row 0 is Header, row 1 is ripgrep, row 2 is git)
    assert_eq!(app.cursors[0], 0);
    app.next_item();
    assert_eq!(app.cursors[0], 1);
    app.next_item();
    assert_eq!(app.cursors[0], 2);

    // 3. Toggle selection (exclude git at row 2)
    app.toggle_selected();
    assert!(!app.user_packages[2].item().unwrap().included);
    assert_eq!(app.count_selected_items(0), 1);

    // 4. Build filter
    let filter = app.build_selection_filter();
    assert!(filter.is_package_included(&PackageName::new("ripgrep").unwrap()));
    assert!(!filter.is_package_included(&PackageName::new("git").unwrap()));
    assert!(!filter.is_package_included(&PackageName::new("linux").unwrap())); // Excluded by default
    assert!(!filter.is_dependency_included(&PackageName::new("glibc").unwrap()));
    assert!(filter.is_skill_included("clean-code"));

    // 5. Test Header toggle (toggle all in category)
    app.cursors[0] = 0; // Header row
    app.toggle_selected();
    // After header toggle when git was excluded and ripgrep was included, any_included was true -> new_state false (all excluded)
    assert!(!app.user_packages[1].item().unwrap().included);
    assert!(!app.user_packages[2].item().unwrap().included);

    // Toggle again -> all included
    app.toggle_selected();
    assert!(app.user_packages[1].item().unwrap().included);
    assert!(app.user_packages[2].item().unwrap().included);

    // 6. Test toggling system package inclusion
    app.current_tab = 1;
    app.cursors[1] = 0; // Header for kernel
    app.toggle_selected(); // Should turn on linux
    assert!(app.system_packages[1].item().unwrap().included);
    let filter_sys = app.build_selection_filter();
    assert!(filter_sys.is_package_included(&PackageName::new("linux").unwrap()));

    // 7. Test toggling dependency inclusion
    app.current_tab = 2;
    app.cursors[2] = 1; // Item glibc
    app.toggle_selected(); // Should turn on glibc
    assert!(app.dependency_packages[1].item().unwrap().included);
    let filter_dep = app.build_selection_filter();
    assert!(filter_dep.is_dependency_included(&PackageName::new("glibc").unwrap()));

    // 6. Custom Directory Addition
    let tmp = tempdir().expect("Failed to create tempdir");
    let custom_p = tmp.path().join("my_custom_folder");
    std::fs::create_dir_all(&custom_p).unwrap();
    std::fs::write(custom_p.join("data.txt"), b"sample").unwrap();

    let res = app.add_custom_directory(custom_p.to_str().unwrap());
    assert!(res.is_ok());
    assert_eq!(app.custom_dirs.len(), 1);
    assert_eq!(app.custom_dirs[0].file_count, 1);

    // 7. Mode toggling and Restore mode state
    assert_eq!(app.mode, adapter_cli::tui::TuiMode::Backup);
    app.toggle_mode();
    assert_eq!(app.mode, adapter_cli::tui::TuiMode::Restore);

    // Add dummy snapshot file
    let snap_path = tmp.path().join("distro-vault-test.tar.zst");
    std::fs::write(&snap_path, b"test content").unwrap();
    app.add_snapshot_file(snap_path.clone(), None);

    assert_eq!(app.snapshots.len(), 1);
    assert_eq!(
        app.selected_snapshot().unwrap().filename,
        "distro-vault-test.tar.zst"
    );
    assert_eq!(app.snapshot_cursor, 0);

    // Test restore options toggle
    assert!(app.restore_install_packages);
    app.toggle_restore_install_packages();
    assert!(!app.restore_install_packages);

    assert!(!app.restore_dry_run);
    app.toggle_restore_dry_run();
    assert!(app.restore_dry_run);

    // Toggle back to Backup
    app.toggle_mode();
    assert_eq!(app.mode, adapter_cli::tui::TuiMode::Backup);
}

#[test]
fn test_options_and_config_modal_interactions() {
    let mut app = TuiApp::new();

    // 1. Options Modal State
    assert!(!app.options_modal.is_open);
    app.toggle_options_modal();
    assert!(app.options_modal.is_open);
    app.toggle_options_modal();
    assert!(!app.options_modal.is_open);

    // 2. Compact View Toggle
    assert!(!app.vault_config.compact_view);
    app.toggle_compact_view();
    assert!(app.vault_config.compact_view);
    app.toggle_compact_view();
    assert!(!app.vault_config.compact_view);

    // 3. Config Modal State
    use adapter_cli::tui::ConfigField;
    use std::path::PathBuf;

    assert!(!app.config_modal.is_open);
    app.toggle_config_modal();
    assert!(app.config_modal.is_open);
    assert_eq!(app.config_modal.focused_field, ConfigField::OutputDir);

    // Navigate fields
    app.config_modal.next_field();
    assert_eq!(app.config_modal.focused_field, ConfigField::InputDirsList);
    app.config_modal.next_field();
    assert_eq!(app.config_modal.focused_field, ConfigField::CompactViewToggle);
    app.config_modal.next_field();
    assert_eq!(app.config_modal.focused_field, ConfigField::SaveAndClose);
    app.config_modal.next_field();
    assert_eq!(app.config_modal.focused_field, ConfigField::OutputDir);
    app.config_modal.prev_field();
    assert_eq!(app.config_modal.focused_field, ConfigField::SaveAndClose);

    // Manipulate config modal values
    let new_out = PathBuf::from("/tmp/custom_backups");
    app.config_modal.set_output_dir(new_out.clone());
    assert_eq!(app.config_modal.pending_output_dir, new_out);

    let new_in = PathBuf::from("/tmp/custom_inputs");
    app.config_modal.add_input_dir(new_in.clone());
    assert!(app.config_modal.pending_input_dirs.contains(&new_in));

    app.config_modal.toggle_compact_view();
    assert!(app.config_modal.pending_compact_view);

    // Apply modal changes
    app.apply_config_modal();
    assert!(!app.config_modal.is_open);
    assert_eq!(app.vault_config.output_dir, new_out);
    assert_eq!(app.default_backups_dir, new_out);
    assert!(app.vault_config.input_dirs.contains(&new_in));
    assert!(app.vault_config.compact_view);
}
