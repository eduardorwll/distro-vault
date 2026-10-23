use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use domain::entities::agent_governance::{AgentGovernance, RuleEntry, SkillEntry};
use domain::errors::DomainError;
use domain::value_objects::SkillCategory;

pub struct AgentScanner;

impl AgentScanner {
    /// Scans the user home for active agent governance, skills, and rules
    pub fn scan(user_home: &Path) -> Result<AgentGovernance, DomainError> {
        let default_dir = user_home.join("Antigravity").to_string_lossy().to_string();
        let mut governance = AgentGovernance::new(default_dir);

        governance.rules = Self::scan_rules(user_home)?;
        governance.skills = Self::scan_skills(user_home)?;

        // Check active cli wrappers
        if user_home.join(".gemini/antigravity-cli/bin/agy").exists() {
            governance.cli_wrappers.push("agy".to_string());
        }

        Ok(governance)
    }

    /// Scans ~/.agents/rules and home rules (AGENTS.md, GEMINI.md)
    pub fn scan_rules(user_home: &Path) -> Result<Vec<RuleEntry>, DomainError> {
        let mut rules = Vec::new();

        // 1. Check root GEMINI.md and AGENTS.md
        let gemini_md = user_home.join("GEMINI.md");
        if gemini_md.exists() {
            rules.push(RuleEntry {
                name: "GEMINI.md".to_string(),
                relative_path: PathBuf::from("GEMINI.md"),
                description: "User guidelines, default directory and system prompts".to_string(),
                is_mandatory: true,
            });
        }

        let agents_md = user_home.join("AGENTS.md");
        if agents_md.exists() {
            rules.push(RuleEntry {
                name: "AGENTS.md".to_string(),
                relative_path: PathBuf::from("AGENTS.md"),
                description: "Engineering protocols, mandatory pre-approval, 5-phase gate".to_string(),
                is_mandatory: true,
            });
        }

        // 2. Check ~/.agents/rules/
        let rules_dir = user_home.join(".agents/rules");
        if rules_dir.exists() && rules_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&rules_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        rules.push(RuleEntry {
                            name: name.clone(),
                            relative_path: PathBuf::from(format!(".agents/rules/{}", name)),
                            description: format!("Modular rule definition: {}", name),
                            is_mandatory: false,
                        });
                    }
                }
            }
        }

        Ok(rules)
    }

    /// Scans ~/.agents/skills/ extracting categories, frontmatter and detecting symlinks
    pub fn scan_skills(user_home: &Path) -> Result<Vec<SkillEntry>, DomainError> {
        let skills_root = user_home.join(".agents/skills");
        if !skills_root.exists() || !skills_root.is_dir() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();
        let entries = fs::read_dir(&skills_root).map_err(|e| DomainError::AgentGovernanceError {
            message: format!("Failed to read skills directory {:?}: {}", skills_root, e),
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            let skill_name = entry.file_name().to_string_lossy().to_string();

            let symlink_meta = fs::symlink_metadata(&path).map_err(|e| DomainError::AgentGovernanceError {
                message: format!("Failed to read metadata for {:?}: {}", path, e),
            })?;

            let is_symlink = symlink_meta.is_symlink();
            let original_target = if is_symlink {
                fs::read_link(&path).ok()
            } else {
                None
            };

            // Resolved path for reading SKILL.md
            let resolved_dir = if let Some(ref target) = original_target {
                if target.is_relative() {
                    skills_root.join(target)
                } else {
                    target.clone()
                }
            } else {
                path.clone()
            };

            let skill_md = resolved_dir.join("SKILL.md");
            let description = if skill_md.exists() {
                Self::extract_skill_description(&skill_md)
            } else {
                None
            };

            let category = SkillCategory::from_name(&skill_name);

            skills.push(SkillEntry {
                name: skill_name.clone(),
                category,
                is_symlink,
                dereferenced: false,
                original_target,
                relative_path: PathBuf::from(format!(".agents/skills/{}", skill_name)),
                description,
            });
        }

        skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(skills)
    }

    /// Extracts description from SKILL.md YAML frontmatter
    fn extract_skill_description(skill_md_path: &Path) -> Option<String> {
        let content = fs::read_to_string(skill_md_path).ok()?;
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return None;
        }

        let after_first = &trimmed[3..];
        let end_idx = after_first.find("---")?;
        let frontmatter = &after_first[..end_idx];

        if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
            if let Some(desc) = val.get("description").and_then(|d| d.as_str()) {
                return Some(desc.trim().to_string());
            }
        }

        None
    }

    /// Recursively copies directory, dereferencing any symlinks to physical files
    pub fn copy_dereferenced(src: &Path, dst: &Path) -> Result<(), DomainError> {
        let canonical_src = src.canonicalize().map_err(|e| DomainError::AgentGovernanceError {
            message: format!("Failed to canonicalize source {:?}: {}", src, e),
        })?;

        if !dst.exists() {
            fs::create_dir_all(dst).map_err(|e| DomainError::AgentGovernanceError {
                message: format!("Failed to create destination {:?}: {}", dst, e),
            })?;
        }

        for entry in WalkDir::new(&canonical_src).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            let rel = match entry_path.strip_prefix(&canonical_src) {
                Ok(r) => r,
                Err(_) => continue,
            };
            if rel.as_os_str().is_empty() {
                continue;
            }

            let target_dest = dst.join(rel);

            // If target_dest already exists as a symlink, unlink it so we can create/write real files
            if let Ok(meta) = fs::symlink_metadata(&target_dest) {
                if meta.is_symlink() {
                    let _ = fs::remove_file(&target_dest);
                }
            }

            if entry_path.is_dir() {
                fs::create_dir_all(&target_dest).map_err(|e| DomainError::AgentGovernanceError {
                    message: format!("Failed to create dir {:?}: {}", target_dest, e),
                })?;
            } else if entry_path.is_file() {
                if let Some(parent) = target_dest.parent() {
                    if let Ok(meta) = fs::symlink_metadata(parent) {
                        if meta.is_symlink() {
                            let _ = fs::remove_file(parent);
                        }
                    }
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| DomainError::AgentGovernanceError {
                            message: format!("Failed to create parent dir {:?}: {}", parent, e),
                        })?;
                    }
                }
                fs::copy(entry_path, &target_dest).map_err(|e| DomainError::AgentGovernanceError {
                    message: format!("Failed to copy file {:?} to {:?}: {}", entry_path, target_dest, e),
                })?;
            }
        }

        Ok(())
    }
}
