use serde::{Deserialize, Serialize};

use crate::value_objects::PackageName;

/// Sources/Managers responsible for providing packages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageSource {
    PacmanNative,
    Aur,
    Flatpak,
    DnfNative,
    AptNative,
    Nix,
    Brew,
    SystemdService,
}

/// Installation scope of a package or service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageScope {
    System,
    User,
}

/// Normalized entry representing an installed or declared software package
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEntry {
    pub name: PackageName,
    pub source: PackageSource,
    pub scope: PackageScope,
    pub version: Option<String>,
    pub description: Option<String>,
}

impl PackageEntry {
    pub fn new(name: PackageName, source: PackageSource, scope: PackageScope) -> Self {
        Self {
            name,
            source,
            scope,
            version: None,
            description: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}
