use serde::{Deserialize, Serialize};
use std::fmt;

use crate::errors::DomainError;

/// Strong newtype for Zen Profile identification (e.g. "7awtlvx8.Default (release)")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProfileId(String);

impl ProfileId {
    pub fn new(id: impl Into<String>) -> Result<Self, DomainError> {
        let s = id.into().trim().to_string();
        if s.is_empty() {
            return Err(DomainError::ValidationError {
                field: "ProfileId",
                reason: "Profile ID cannot be empty".to_string(),
            });
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Normalized package identifier (e.g. "ripgrep", "zen-browser-bin")
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PackageName(String);

impl PackageName {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let s = name.into().trim().to_string();
        if s.is_empty() {
            return Err(DomainError::ValidationError {
                field: "PackageName",
                reason: "Package name cannot be empty".to_string(),
            });
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Linux distribution families recognized by the package and system engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistroFamily {
    Arch,
    Fedora,
    Debian,
    OpenSuse,
    Unknown,
}

impl DistroFamily {
    pub fn from_id(id: &str, id_like: Option<&str>) -> Self {
        let id_lower = id.to_lowercase();
        let like_lower = id_like.unwrap_or("").to_lowercase();

        if id_lower == "arch"
            || id_lower == "omarchy"
            || id_lower == "cachyos"
            || id_lower == "endeavouros"
            || id_lower == "manjaro"
            || like_lower.contains("arch")
        {
            DistroFamily::Arch
        } else if id_lower == "fedora" || id_lower == "rhel" || like_lower.contains("fedora") {
            DistroFamily::Fedora
        } else if id_lower == "debian"
            || id_lower == "ubuntu"
            || id_lower == "pop"
            || id_lower == "mint"
            || like_lower.contains("debian")
        {
            DistroFamily::Debian
        } else if id_lower.contains("suse") || like_lower.contains("suse") {
            DistroFamily::OpenSuse
        } else {
            DistroFamily::Unknown
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DistroFamily::Arch => "arch",
            DistroFamily::Fedora => "fedora",
            DistroFamily::Debian => "debian",
            DistroFamily::OpenSuse => "opensuse",
            DistroFamily::Unknown => "unknown",
        }
    }
}

impl fmt::Display for DistroFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// SHA-256 Checksum representation for snapshot integrity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Checksum(String);

impl Checksum {
    pub fn new(hash: impl Into<String>) -> Result<Self, DomainError> {
        let s = hash.into().trim().to_lowercase();
        if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DomainError::ValidationError {
                field: "Checksum",
                reason: "Checksum must be a valid 64-character hex string (SHA-256)".to_string(),
            });
        }
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// POSIX file permission mode (e.g. 0o700, 0o600)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnixMode(u32);

impl UnixMode {
    pub const SECURE_DIR: UnixMode = UnixMode(0o700);
    pub const SECURE_PRIVATE_KEY: UnixMode = UnixMode(0o600);
    pub const SECURE_PUBLIC_KEY: UnixMode = UnixMode(0o644);
    pub const EXECUTABLE_FILE: UnixMode = UnixMode(0o755);

    pub fn new(mode: u32) -> Self {
        Self(mode & 0o777)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }

    pub fn is_executable(&self) -> bool {
        (self.0 & 0o111) != 0
    }

    pub fn is_strictly_private(&self) -> bool {
        (self.0 & 0o077) == 0
    }
}

impl fmt::Display for UnixMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04o}", self.0)
    }
}

/// Categories of AI Agent Skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    DesignPattern,
    Architecture,
    Security,
    Testing,
    Process,
    General,
}

impl SkillCategory {
    pub fn from_name(name: &str) -> Self {
        let n = name.to_lowercase();
        if n.contains("design") || n.contains("clean-code") || n.contains("philosophy") {
            SkillCategory::DesignPattern
        } else if n.contains("arch") || n.contains("data-intensive") || n.contains("adr") {
            SkillCategory::Architecture
        } else if n.contains("sec") || n.contains("doubt") || n.contains("hardening") {
            SkillCategory::Security
        } else if n.contains("test") || n.contains("debug") || n.contains("tdd") || n.contains("quality") {
            SkillCategory::Testing
        } else if n.contains("plan") || n.contains("incremental") || n.contains("shipping") {
            SkillCategory::Process
        } else {
            SkillCategory::General
        }
    }
}
