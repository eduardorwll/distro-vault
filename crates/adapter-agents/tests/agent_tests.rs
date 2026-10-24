use std::fs;
use std::path::Path;
use tempfile::tempdir;

use adapter_agents::LocalAgentAdapter;
use domain::ports::AgentPort;
use domain::value_objects::SkillCategory;

#[test]
fn test_agent_scanner_reads_rules_and_frontmatter() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let home = tmp.path();

    // 1. Create GEMINI.md and AGENTS.md
    fs::write(
        home.join("GEMINI.md"),
        "# Guidelines\nLocal default: ~/Antigravity/",
    )
    .unwrap();
    fs::write(
        home.join("AGENTS.md"),
        "# Protocols\nMandatory pre-approval",
    )
    .unwrap();

    // 2. Create skills with SKILL.md containing YAML frontmatter
    let skills_dir = home.join(".agents/skills");
    let tdd_dir = skills_dir.join("tdd");
    fs::create_dir_all(&tdd_dir).unwrap();

    let tdd_skill_md = r#"---
name: tdd
description: Test-driven development red-green-refactor workflow.
---
# TDD Instructions
"#;
    fs::write(tdd_dir.join("SKILL.md"), tdd_skill_md).unwrap();

    let adapter = LocalAgentAdapter::new();
    let governance = adapter.scan_governance(home).expect("Scan failed");

    assert_eq!(governance.rules.len(), 2);
    assert!(governance.rules.iter().any(|r| r.name == "GEMINI.md"));
    assert!(governance.rules.iter().any(|r| r.name == "AGENTS.md"));

    assert_eq!(governance.skills.len(), 1);
    let skill = &governance.skills[0];
    assert_eq!(skill.name, "tdd");
    assert_eq!(skill.category, SkillCategory::Testing);
    assert_eq!(
        skill.description.as_deref(),
        Some("Test-driven development red-green-refactor workflow.")
    );
    assert!(!skill.is_symlink);
}

#[cfg(unix)]
#[test]
fn test_agent_symlink_dereferencing() {
    use std::os::unix::fs::symlink;

    let tmp = tempdir().expect("Failed to create tempdir");
    let home = tmp.path();

    // External target directory simulating /usr/share/omarchy/
    let external_omarchy = tmp.path().join("external_share/omarchy");
    fs::create_dir_all(&external_omarchy).unwrap();
    fs::write(
        external_omarchy.join("SKILL.md"),
        "---\nname: omarchy\ndescription: Omarchy desktop skill.\n---\nBody",
    )
    .unwrap();
    fs::write(external_omarchy.join("config.sh"), "#!/bin/bash\necho ok").unwrap();

    // Create symlink ~/.agents/skills/omarchy -> external_omarchy
    let skills_dir = home.join(".agents/skills");
    fs::create_dir_all(&skills_dir).unwrap();
    let symlink_path = skills_dir.join("omarchy");
    symlink(&external_omarchy, &symlink_path).unwrap();

    let adapter = LocalAgentAdapter::new();
    let governance = adapter.scan_governance(home).expect("Scan failed");

    assert_eq!(governance.skills.len(), 1);
    let skill = &governance.skills[0];
    assert!(skill.is_symlink);

    // Export with dereference_symlinks = true
    let export_dir = tmp.path().join("export");
    adapter
        .export_governance(&governance, &export_dir, true)
        .expect("Export failed");

    let exported_omarchy = export_dir.join(".agents/skills/omarchy");
    assert!(exported_omarchy.exists());
    assert!(
        !exported_omarchy.is_symlink(),
        "Exported skill must be a real directory, not a symlink"
    );
    assert!(exported_omarchy.join("SKILL.md").exists());
    assert!(exported_omarchy.join("config.sh").exists());
}

#[test]
fn test_real_system_agent_scanning() {
    let home = Path::new("/home/eduardorwll");
    if home.join(".agents/skills").exists() {
        let adapter = LocalAgentAdapter::new();
        let governance = adapter
            .scan_governance(home)
            .expect("Scan of real system failed");

        assert!(
            !governance.skills.is_empty(),
            "Expected real skills to be discovered"
        );
        assert!(governance.skills.len() >= 10);
        assert!(governance.rules.iter().any(|r| r.name == "GEMINI.md"));
    }
}
