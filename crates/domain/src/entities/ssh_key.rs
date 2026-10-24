use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Type classification for SSH files stored in ~/.ssh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SshFileType {
    PrivateKey,
    PublicKey,
    Config,
    AuthorizedKeys,
    KnownHosts,
    Certificate,
    Other,
}

impl SshFileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PrivateKey => "Chave Privada",
            Self::PublicKey => "Chave Pública",
            Self::Config => "Configuração SSH",
            Self::AuthorizedKeys => "Chaves Autorizadas",
            Self::KnownHosts => "Hosts Conhecidos",
            Self::Certificate => "Certificado SSH",
            Self::Other => "Arquivo Auxiliar",
        }
    }

    /// Strict POSIX file permission mode recommended by OpenSSH specifications
    pub fn default_posix_mode(&self) -> u32 {
        match self {
            Self::PrivateKey => 0o600,
            Self::PublicKey => 0o644,
            Self::Config => 0o600,
            Self::AuthorizedKeys => 0o600,
            Self::KnownHosts => 0o644,
            Self::Certificate => 0o644,
            Self::Other => 0o600,
        }
    }
}

/// Metadata describing an SSH key or auxiliary configuration entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshKeyEntry {
    pub file_name: String,
    pub relative_path: PathBuf,
    pub file_type: SshFileType,
    pub size_bytes: u64,
    pub posix_mode: u32,
}

impl SshKeyEntry {
    pub fn new(
        file_name: impl Into<String>,
        relative_path: impl Into<PathBuf>,
        file_type: SshFileType,
        size_bytes: u64,
        posix_mode: u32,
    ) -> Self {
        Self {
            file_name: file_name.into(),
            relative_path: relative_path.into(),
            file_type,
            size_bytes,
            posix_mode,
        }
    }
}
