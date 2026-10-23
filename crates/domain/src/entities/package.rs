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

/// Detailed functional category for a package
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PackageCategory {
    // User Scope
    DesktopApp,
    CliTool,
    Development,
    FontsThemes,
    UserOther,

    // Low-Level System Scope
    KernelFirmware,
    BaseBoot,
    DriversHardware,
    AudioVideoStack,
    NetworkDaemons,
    FileSystems,
    CompositorPlumbing,
    SystemOther,

    // Dependency Scope (Installed indirectly as dependencies)
    CoreLibrary,
    GraphicsMediaLib,
    LanguageRuntimeLib,
    ToolchainBuildLib,
    GeneralDependency,

    #[default]
    Other,
}

impl PackageCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DesktopApp => "App Desktop",
            Self::CliTool => "Ferramenta CLI",
            Self::Development => "Dev & Runtime",
            Self::FontsThemes => "Fontes & Temas",
            Self::UserOther => "App do Usuário",
            Self::KernelFirmware => "Kernel & Firmware",
            Self::BaseBoot => "Sistema Base & Boot",
            Self::DriversHardware => "Drivers & GPU",
            Self::AudioVideoStack => "Áudio & Mídia Core",
            Self::NetworkDaemons => "Rede & Daemons",
            Self::FileSystems => "Sistemas de Arquivos",
            Self::CompositorPlumbing => "Compositor & Display",
            Self::SystemOther => "Sistema Low-Level",
            Self::CoreLibrary => "Bibliotecas Core C/C++",
            Self::GraphicsMediaLib => "Gráficos & Multimídia",
            Self::LanguageRuntimeLib => "Linguagens & Runtimes",
            Self::ToolchainBuildLib => "Compiladores & Build Tools",
            Self::GeneralDependency => "Dependências Gerais",
            Self::Other => "Geral",
        }
    }

    /// Classifies a dependency package name into a PackageCategory
    pub fn classify_dependency(name: &str) -> PackageCategory {
        let n = name.to_lowercase();
        if n.starts_with("python")
            || n.starts_with("perl")
            || n.starts_with("ruby")
            || n.starts_with("lua")
            || n.contains("dotnet")
        {
            PackageCategory::LanguageRuntimeLib
        } else if n.contains("ffmpeg")
            || n.contains("gstreamer")
            || n.contains("gst-")
            || n.contains("libx")
            || n.contains("wayland")
            || n.contains("vulkan")
            || n.contains("mesa")
            || n.contains("cairo")
            || n.contains("pango")
            || n.contains("pipewire")
            || n.contains("alsa")
            || n.contains("font")
            || n.contains("codec")
            || n.contains("aom")
            || n.contains("dav1d")
            || n.contains("harfbuzz")
            || n.contains("pixman")
        {
            PackageCategory::GraphicsMediaLib
        } else if n.contains("gcc")
            || n.contains("llvm")
            || n.contains("clang")
            || n.contains("binutils")
            || n.contains("cmake")
            || n.contains("make")
            || n.contains("patch")
            || n.contains("pkgconf")
            || n.contains("bison")
            || n.contains("flex")
            || n.contains("m4")
            || n.contains("autoconf")
            || n.contains("automake")
        {
            PackageCategory::ToolchainBuildLib
        } else if n.starts_with("lib")
            || n.contains("glibc")
            || n.contains("ssl")
            || n.contains("xml")
            || n.contains("icu")
            || n.contains("zlib")
            || n.contains("pcre")
            || n.contains("sqlite")
            || n.contains("curl")
            || n.contains("ffi")
            || n.contains("dbus")
            || n.contains("json")
            || n.contains("readline")
            || n.contains("archive")
            || n.contains("zstd")
            || n.contains("acl")
            || n.contains("attr")
            || n.contains("audit")
        {
            PackageCategory::CoreLibrary
        } else {
            PackageCategory::GeneralDependency
        }
    }

    /// Classifies a package name and source into (PackageScope, PackageCategory)
    pub fn classify(name: &str, source: PackageSource) -> (PackageScope, PackageCategory) {
        if source == PackageSource::Flatpak {
            return (PackageScope::User, PackageCategory::DesktopApp);
        }

        let n = name.to_lowercase();

        // 1. Kernel & Firmware
        if n.starts_with("linux")
            || n.contains("firmware")
            || n.contains("ucode")
            || n == "wireless-regdb"
            || n.contains("kernel-modules")
        {
            return (PackageScope::System, PackageCategory::KernelFirmware);
        }

        // 2. Base & Boot
        if n == "base"
            || n == "base-devel"
            || n == "sudo"
            || n == "mkinitcpio"
            || n == "limine"
            || n == "efibootmgr"
            || n == "edk2-ovmf"
            || n == "grub"
            || n == "systemd"
            || n == "systemd-sysvcompat"
            || n == "glibc"
            || n == "coreutils"
            || n.contains("keyring")
        {
            return (PackageScope::System, PackageCategory::BaseBoot);
        }

        // 3. Filesystems & Storage Low-Level
        if n.contains("btrfs")
            || n.contains("dosfstools")
            || n.contains("exfat")
            || n.contains("fuse")
            || n.contains("zram")
            || n.contains("e2fsprogs")
            || n.contains("parted")
            || n.contains("lvm")
            || n.contains("mdadm")
        {
            return (PackageScope::System, PackageCategory::FileSystems);
        }

        // 4. Drivers & Hardware & GPU
        if n.contains("vulkan")
            || n.contains("mesa")
            || n.contains("radeon")
            || n.contains("nvidia")
            || n.contains("intel")
            || n.contains("bluez")
            || n.contains("bolt")
            || n.contains("brightnessctl")
            || n.contains("ddcutil")
            || n.contains("libinput")
            || n.contains("xf86-")
        {
            return (PackageScope::System, PackageCategory::DriversHardware);
        }

        // 5. Audio & Media Core Stack
        if n.contains("pipewire")
            || n.contains("wireplumber")
            || n.contains("alsa")
            || n.contains("libpulse")
            || n.contains("gstreamer")
            || n.contains("gst-plugin")
        {
            return (PackageScope::System, PackageCategory::AudioVideoStack);
        }

        // 6. Network, Security & System Daemons
        if n.contains("networkmanager")
            || n.contains("tailscale")
            || n.contains("ufw")
            || n.contains("cups")
            || n.contains("sddm")
            || n.contains("gdm")
            || n.contains("lightdm")
            || n.contains("power-profiles")
            || n.contains("gnome-keyring")
            || n.contains("passt")
            || n.contains("swtpm")
            || n.contains("dhcp")
            || n.contains("iptables")
            || n.contains("nftables")
            || n.contains("avahi")
            || n.contains("nss-mdns")
        {
            return (PackageScope::System, PackageCategory::NetworkDaemons);
        }

        // 7. Compositor Plumbing & Wayland core
        if n.contains("hyprland")
            || n.contains("hyprpicker")
            || n.contains("hyprsunset")
            || n.contains("uwsm")
            || n.contains("xdg-desktop-portal")
            || n.contains("wayland")
            || n.contains("xwayland")
            || n.contains("slurp")
            || n.contains("grim")
            || n.contains("wl-clipboard")
            || n.contains("wtype")
            || n.contains("quickshell")
        {
            return (PackageScope::System, PackageCategory::CompositorPlumbing);
        }

        // 8. Fonts & Themes
        if n.starts_with("ttf-")
            || n.starts_with("noto-fonts")
            || n.starts_with("woff")
            || n.contains("font")
            || n.contains("theme")
            || n.contains("icon")
        {
            return (PackageScope::User, PackageCategory::FontsThemes);
        }

        // 9. Development & Toolchains
        if n.contains("docker")
            || n == "git"
            || n == "bun"
            || n == "clang"
            || n == "llvm"
            || n.starts_with("rust")
            || n.starts_with("python-")
            || n.starts_with("ruby")
            || n.starts_with("lua")
            || n.contains("dotnet")
            || n == "godot"
            || n.contains("qemu")
            || n.contains("tree-sitter")
            || n.contains("mise")
            || n == "yay"
            || n == "paru"
            || n.contains("wine")
            || n.contains("winetricks")
            || n.contains("compiler")
            || n.contains("mariadb")
            || n.contains("postgresql")
            || n.contains("sqlite")
        {
            return (PackageScope::User, PackageCategory::Development);
        }

        // 10. CLI User Tools
        if n == "bat"
            || n == "btop"
            || n == "eza"
            || n == "fd"
            || n == "fzf"
            || n == "ripgrep"
            || n == "jq"
            || n == "fastfetch"
            || n == "lazygit"
            || n == "lazydocker"
            || n == "dua-cli"
            || n == "zoxide"
            || n == "starship"
            || n == "tmux"
            || n == "tldr"
            || n == "yt-dlp"
            || n == "inxi"
            || n == "plocate"
            || n == "pv"
            || n == "whois"
            || n == "socat"
            || n == "qrencode"
            || n == "htop"
            || n == "fish"
            || n == "zsh"
            || n == "neovim"
            || n == "vim"
            || n == "foot"
            || n == "kitty"
            || n == "alacritty"
            || n == "ghostty"
        {
            return (PackageScope::User, PackageCategory::CliTool);
        }

        // 11. Known User Desktop Applications
        if n.contains("discord")
            || n.contains("chromium")
            || n.contains("zen-browser")
            || n.contains("firefox")
            || n.contains("obsidian")
            || n.contains("vlc")
            || n.contains("mpv")
            || n.contains("qbittorrent")
            || n.contains("kdenlive")
            || n.contains("krita")
            || n.contains("inkscape")
            || n.contains("gimp")
            || n.contains("pinta")
            || n.contains("evince")
            || n.contains("okular")
            || n.contains("xournalpp")
            || n.contains("localsend")
            || n.contains("drawy")
            || n.contains("tenacity")
            || n.contains("easyeffects")
            || n.contains("obs-studio")
            || n.contains("steam")
            || n.contains("lutris")
            || n.contains("dbeaver")
            || n.contains("code")
            || n.contains("nautilus")
            || n.contains("sushi")
            || n.contains("imv")
            || n.contains("calc")
            || n.contains("spotify")
            || n.contains("telegram")
            || n.contains("lmstudio")
        {
            return (PackageScope::User, PackageCategory::DesktopApp);
        }

        // Default heuristic: User package
        (PackageScope::User, PackageCategory::UserOther)
    }
}

/// Normalized entry representing an installed or declared software package
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEntry {
    pub name: PackageName,
    pub source: PackageSource,
    pub scope: PackageScope,
    pub category: PackageCategory,
    pub is_dependency: bool,
    pub version: Option<String>,
    pub description: Option<String>,
}

impl PackageEntry {
    pub fn new(name: PackageName, source: PackageSource, scope: PackageScope) -> Self {
        let category = match scope {
            PackageScope::User => PackageCategory::UserOther,
            PackageScope::System => PackageCategory::SystemOther,
        };
        Self {
            name,
            source,
            scope,
            category,
            is_dependency: false,
            version: None,
            description: None,
        }
    }

    pub fn with_category(mut self, category: PackageCategory) -> Self {
        self.category = category;
        self
    }

    pub fn as_dependency(mut self, is_dep: bool) -> Self {
        self.is_dependency = is_dep;
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}
