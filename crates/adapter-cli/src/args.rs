use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "distro-vault",
    version,
    about = "High-performance declarative Linux backup, migration and recovery system in Rust",
    long_about = "distro-vault provides full declarative system backup, cross-distro hopping (Arch/Omarchy -> Fedora/Debian),\nZen Browser session preservation (mozLz40 codec, Spaces, tabs, CityHash64 normalizer),\nAppImages, package manifests and AI Agent Governance."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a declarative backup snapshot (.tar.zst)
    Backup(BackupArgs),

    /// Restore or migrate system from a snapshot
    Restore(RestoreArgs),

    /// Inspect a snapshot manifest without extracting the archive
    Inspect(InspectArgs),

    /// Verify the cryptographic checksum and structural integrity of a snapshot
    Verify(VerifyArgs),
}

#[derive(Debug, Args, Clone)]
pub struct BackupArgs {
    /// Output archive path (default: ./backups/distro-vault-<timestamp>.tar.zst)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Launch interactive Ratatui TUI for item selection and exclusions
    #[arg(short, long, default_value_t = false)]
    pub interactive: bool,

    /// Package name to exclude from backup (can be specified multiple times)
    #[arg(long = "exclude-package")]
    pub exclude_packages: Vec<String>,

    /// AI Agent skill name to exclude (can be specified multiple times)
    #[arg(long = "exclude-skill")]
    pub exclude_skills: Vec<String>,

    /// SSH key filename to exclude from backup (can be specified multiple times)
    #[arg(long = "exclude-ssh-key")]
    pub exclude_ssh_keys: Vec<String>,

    /// Do not include any SSH keys in the backup snapshot
    #[arg(long = "no-ssh", default_value_t = false)]
    pub no_ssh: bool,

    /// Extra custom directory path to include and compress (can be specified multiple times)
    #[arg(long = "add-dir")]
    pub add_dirs: Vec<PathBuf>,

    /// Simulate backup workflow without writing to disk
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Override Zen Browser configuration root (default: ~/.config/zen)
    #[arg(long)]
    pub config_root: Option<PathBuf>,

    /// Override user home directory (default: $HOME)
    #[arg(long)]
    pub user_home: Option<PathBuf>,
}

#[derive(Debug, Args, Clone)]
pub struct RestoreArgs {
    /// Path to the .tar.zst snapshot archive to restore (if omitted, opens interactive TUI)
    pub archive: Option<PathBuf>,

    /// Automatically install declarative packages through distro backends
    #[arg(long, default_value_t = false)]
    pub install_packages: bool,

    /// Simulate restoration without modifying filesystem or installing packages
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Override target user home directory
    #[arg(long)]
    pub target_home: Option<PathBuf>,

    /// Override target Zen Browser configuration root
    #[arg(long)]
    pub target_config: Option<PathBuf>,

    /// Override target base folder for restored custom directories
    #[arg(long)]
    pub target_custom: Option<PathBuf>,

    /// Fast-path restoration using an already extracted staging directory (skips archive decompression)
    #[arg(long)]
    pub from_staging: Option<PathBuf>,
}

#[derive(Debug, Args, Clone)]
pub struct InspectArgs {
    /// Path to the .tar.zst snapshot archive to inspect
    pub archive: PathBuf,

    /// Output full manifest details in raw JSON format
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct VerifyArgs {
    /// Path to the .tar.zst snapshot archive to verify
    pub archive: PathBuf,
}
