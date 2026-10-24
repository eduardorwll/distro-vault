use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;

use adapter_cli::args::{
    BackupArgs, Cli, Commands, InspectArgs, RestoreArgs, SliceArgs, VerifyArgs,
};
use adapter_cli::di::AppContainer;
use adapter_cli::tui::{run_tui, TuiAction, TuiApp, TuiItem, TuiMode, TuiRow};
use adapter_storage::CustomDirsHandler;
use application::{BackupRequest, RestoreRequest, SliceSnapshotRequest};
use domain::entities::custom_directory::CustomDirectoryEntry;
use domain::entities::package::{PackageCategory, PackageScope};
use domain::entities::selection_filter::SelectionFilter;
use domain::ports::{AgentPort, BrowserPort, SshPort, StoragePort};
use domain::value_objects::{PackageName, SkillCategory};

fn main() {
    let cli = Cli::parse();
    let container = AppContainer::bootstrap();

    let result = match cli.command {
        Some(Commands::Backup(args)) => handle_backup(&container, args),
        Some(Commands::Restore(args)) => handle_restore(&container, args),
        Some(Commands::Inspect(args)) => handle_inspect(&container, args),
        Some(Commands::Slice(args)) => handle_slice(&container, args),
        Some(Commands::Verify(args)) => handle_verify(&container, args),
        None => {
            // Default action: launch interactive TUI
            let home = default_home();
            let config = home.join(".config/zen");
            handle_interactive(&container, TuiMode::Backup, home, config, Vec::new(), None)
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("Erro:").red().bold(), e);
        std::process::exit(1);
    }
}

fn handle_backup(
    container: &AppContainer,
    args: BackupArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let home = args.user_home.unwrap_or_else(default_home);
    let config = args.config_root.unwrap_or_else(|| home.join(".config/zen"));

    if args.interactive {
        handle_interactive(
            container,
            TuiMode::Backup,
            home,
            config,
            args.add_dirs,
            None,
        )
    } else {
        let output_archive = args.output.unwrap_or_else(|| {
            let backups_dir = default_backups_dir();
            let _ = std::fs::create_dir_all(&backups_dir);
            let timestamp = chrono_timestamp_slug();
            backups_dir.join(format!("distro-vault-{}.tar.zst", timestamp))
        });

        let mut filter = SelectionFilter::new();
        for p in &args.exclude_packages {
            if let Ok(name) = PackageName::new(p) {
                filter.toggle_package(name);
            }
        }
        for s in &args.exclude_skills {
            filter.toggle_skill(s.clone());
        }

        if args.no_ssh {
            if let Ok(keys) = container.ssh_adapter.scan_ssh_keys(&home) {
                for k in keys {
                    filter.exclude_ssh_key(k.file_name);
                }
            }
        } else {
            for k in &args.exclude_ssh_keys {
                filter.exclude_ssh_key(k.clone());
            }
        }

        let mut custom_dirs = Vec::new();
        for d in &args.add_dirs {
            if let Ok(entry) = CustomDirsHandler::inspect_directory(d, None) {
                custom_dirs.push(entry);
            }
        }

        execute_backup(
            container,
            &home,
            &config,
            output_archive,
            filter,
            custom_dirs,
            args.dry_run,
        )
    }
}

fn handle_restore(
    container: &AppContainer,
    args: RestoreArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let home = args.target_home.unwrap_or_else(default_home);
    let config = args
        .target_config
        .unwrap_or_else(|| home.join(".config/zen"));

    if args.archive.is_some() || args.from_staging.is_some() {
        let archive = args
            .archive
            .unwrap_or_else(|| PathBuf::from("snapshot.tar.zst"));
        let params = RestoreExecutionParams {
            archive: &archive,
            config: &config,
            home: &home,
            target_custom: args.target_custom,
            from_staging: args.from_staging,
            install_packages: args.install_packages,
            dry_run: args.dry_run,
            selection_filter: None,
        };
        execute_restore(container, params)
    } else {
        // No archive passed -> launch interactive TUI directly in Restore mode!
        handle_interactive(
            container,
            TuiMode::Restore,
            home,
            config,
            Vec::new(),
            args.target_custom,
        )
    }
}

fn handle_interactive(
    container: &AppContainer,
    initial_mode: TuiMode,
    home: PathBuf,
    config: PathBuf,
    add_dirs: Vec<PathBuf>,
    target_custom: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tui_app = TuiApp::new();
    tui_app.set_mode(initial_mode);

    // 1. Scan packages (User vs Low-Level System vs Dependencies)
    let (user_pkgs, sys_pkgs, dep_pkgs) = scan_package_categories(container);
    tui_app.user_packages = user_pkgs;
    tui_app.system_packages = sys_pkgs;
    tui_app.dependency_packages = dep_pkgs;

    // 2. Scan skills by category
    tui_app.skills = scan_agent_skills(container, &home);

    // 3. Scan Zen spaces
    tui_app.zen_spaces = scan_zen_spaces(container, &config);

    // 4. Scan registered SSH keys (~/.ssh)
    tui_app.ssh_keys = scan_ssh_keys(container, &home);

    // 5. Initial custom dirs from CLI flags
    for dir in &add_dirs {
        let _ = tui_app.add_custom_directory(&dir.to_string_lossy());
    }

    // 6. Discover existing backups in ./backups/ and fallback ~/Antigravity/backups/
    let backups_dir = default_backups_dir();
    tui_app.default_backups_dir = backups_dir.clone();
    populate_snapshots(&mut tui_app, container, &home, &backups_dir);

    // Run TUI loop
    match run_tui(tui_app)? {
        Some(TuiAction::Backup {
            filter,
            custom_dirs,
        }) => {
            let output_archive = {
                let _ = std::fs::create_dir_all(&backups_dir);
                let timestamp = chrono_timestamp_slug();
                backups_dir.join(format!("distro-vault-{}.tar.zst", timestamp))
            };
            execute_backup(
                container,
                &home,
                &config,
                output_archive,
                *filter,
                custom_dirs,
                false,
            )
        }
        Some(TuiAction::Restore {
            archive_path,
            from_staging,
            install_packages,
            dry_run,
            selection_filter,
        }) => {
            let params = RestoreExecutionParams {
                archive: &archive_path,
                config: &config,
                home: &home,
                target_custom,
                from_staging,
                install_packages,
                dry_run,
                selection_filter: selection_filter.map(|f| *f),
            };
            execute_restore(container, params)
        }
        Some(TuiAction::Slice {
            source_archive,
            output_archive,
            selection_filter,
        }) => {
            execute_slice(
                container,
                &source_archive,
                output_archive,
                *selection_filter,
                None,
            )
        }
        None => {
            println!(
                "{}",
                style("Operação cancelada pelo usuário na TUI.").yellow()
            );
            Ok(())
        }
    }
}

fn scan_package_categories(container: &AppContainer) -> (Vec<TuiRow>, Vec<TuiRow>, Vec<TuiRow>) {
    let mut user_desktop = Vec::new();
    let mut user_cli = Vec::new();
    let mut user_dev = Vec::new();
    let mut user_fonts = Vec::new();
    let mut user_other = Vec::new();

    let mut sys_kernel = Vec::new();
    let mut sys_base = Vec::new();
    let mut sys_drivers = Vec::new();
    let mut sys_audio = Vec::new();
    let mut sys_fs = Vec::new();
    let mut sys_net = Vec::new();
    let mut sys_compositor = Vec::new();
    let mut sys_distro = Vec::new();
    let mut sys_other = Vec::new();

    let mut dep_core = Vec::new();
    let mut dep_media = Vec::new();
    let mut dep_runtimes = Vec::new();
    let mut dep_build = Vec::new();
    let mut dep_other = Vec::new();

    for port in &container.package_ports {
        // Explicit packages
        if let Ok(pkgs) = port.query_installed() {
            for p in pkgs {
                let is_user = p.scope == PackageScope::User;
                let item = TuiItem {
                    id: p.name.as_str().to_string(),
                    name: p.name.as_str().to_string(),
                    details: format!(
                        "{:?} • {}",
                        p.source,
                        p.version.as_deref().unwrap_or("latest")
                    ),
                    included: is_user,
                };
                if is_user {
                    match p.category {
                        PackageCategory::DesktopApp => user_desktop.push(item),
                        PackageCategory::CliTool => user_cli.push(item),
                        PackageCategory::Development => user_dev.push(item),
                        PackageCategory::FontsThemes => user_fonts.push(item),
                        _ => user_other.push(item),
                    }
                } else {
                    match p.category {
                        PackageCategory::DistroBase => sys_distro.push(item),
                        PackageCategory::KernelFirmware => sys_kernel.push(item),
                        PackageCategory::BaseBoot => sys_base.push(item),
                        PackageCategory::DriversHardware => sys_drivers.push(item),
                        PackageCategory::AudioVideoStack => sys_audio.push(item),
                        PackageCategory::FileSystems => sys_fs.push(item),
                        PackageCategory::NetworkDaemons => sys_net.push(item),
                        PackageCategory::CompositorPlumbing => sys_compositor.push(item),
                        _ => sys_other.push(item),
                    }
                }
            }
        }

        // Dependencies (installed indirectly)
        if let Ok(deps) = port.query_dependencies() {
            for p in deps {
                let item = TuiItem {
                    id: p.name.as_str().to_string(),
                    name: p.name.as_str().to_string(),
                    details: format!(
                        "{:?} • {}",
                        p.source,
                        p.version.as_deref().unwrap_or("latest")
                    ),
                    included: false,
                };
                match p.category {
                    PackageCategory::CoreLibrary => dep_core.push(item),
                    PackageCategory::GraphicsMediaLib => dep_media.push(item),
                    PackageCategory::LanguageRuntimeLib => dep_runtimes.push(item),
                    PackageCategory::ToolchainBuildLib => dep_build.push(item),
                    _ => dep_other.push(item),
                }
            }
        }
    }

    let user_rows = TuiApp::group_into_rows(vec![
        ("desktop".into(), "APPS DESKTOP".into(), user_desktop),
        ("cli".into(), "FERRAMENTAS CLI".into(), user_cli),
        ("dev".into(), "DESENVOLVIMENTO & RUNTIMES".into(), user_dev),
        ("fonts".into(), "FONTES & TEMAS".into(), user_fonts),
        ("user_other".into(), "DEMAIS APLICATIVOS".into(), user_other),
    ]);

    let sys_rows = TuiApp::group_into_rows(vec![
        (
            "distro".into(),
            "PACOTES BASE DA DISTRIBUIÇÃO (PRÉ-INSTALADOS)".into(),
            sys_distro,
        ),
        ("kernel".into(), "KERNEL & FIRMWARE".into(), sys_kernel),
        ("base".into(), "BASE & BOOTLOADER".into(), sys_base),
        ("drivers".into(), "DRIVERS & HARDWARE".into(), sys_drivers),
        ("audio".into(), "ÁUDIO & MULTIMÍDIA CORE".into(), sys_audio),
        ("fs".into(), "SISTEMAS DE ARQUIVOS".into(), sys_fs),
        ("net".into(), "REDE, SEGURANÇA & DAEMONS".into(), sys_net),
        (
            "compositor".into(),
            "COMPOSITOR & DISPLAY SERVER".into(),
            sys_compositor,
        ),
        (
            "sys_other".into(),
            "OUTROS DO SISTEMA LOW-LEVEL".into(),
            sys_other,
        ),
    ]);

    let dep_rows = TuiApp::group_into_rows(vec![
        (
            "core_libs".into(),
            "BIBLIOTECAS CORE C/C++".into(),
            dep_core,
        ),
        (
            "media_libs".into(),
            "GRÁFICOS, CODECS & MULTIMÍDIA".into(),
            dep_media,
        ),
        (
            "lang_libs".into(),
            "LINGUAGENS, RUNTIMES & BINDINGS".into(),
            dep_runtimes,
        ),
        (
            "build_libs".into(),
            "COMPILADORES & BUILD TOOLS".into(),
            dep_build,
        ),
        (
            "other_deps".into(),
            "DEMAIS DEPENDÊNCIAS DO SISTEMA".into(),
            dep_other,
        ),
    ]);

    (user_rows, sys_rows, dep_rows)
}

fn scan_agent_skills(container: &AppContainer, home: &Path) -> Vec<TuiRow> {
    if let Ok(gov) = container.agent_adapter.scan_governance(home) {
        let mut skill_arch = Vec::new();
        let mut skill_design = Vec::new();
        let mut skill_sec = Vec::new();
        let mut skill_test = Vec::new();
        let mut skill_proc = Vec::new();
        let mut skill_gen = Vec::new();

        for s in gov.skills {
            let item = TuiItem {
                id: s.name.clone(),
                name: s.name.clone(),
                details: s
                    .description
                    .unwrap_or_else(|| "Habilidade de IA".to_string()),
                included: true,
            };
            match s.category {
                SkillCategory::Architecture => skill_arch.push(item),
                SkillCategory::DesignPattern => skill_design.push(item),
                SkillCategory::Security => skill_sec.push(item),
                SkillCategory::Testing => skill_test.push(item),
                SkillCategory::Process => skill_proc.push(item),
                SkillCategory::General => skill_gen.push(item),
            }
        }

        TuiApp::group_into_rows(vec![
            ("arch".into(), "ARQUITETURA & MODELAGEM".into(), skill_arch),
            ("design".into(), "DESIGN & CLEAN CODE".into(), skill_design),
            ("sec".into(), "SEGURANÇA & QUALITY GATES".into(), skill_sec),
            ("testing".into(), "TESTES & DIAGNÓSTICO".into(), skill_test),
            ("proc".into(), "PROCESSOS & FLUXOS".into(), skill_proc),
            (
                "general".into(),
                "DEMAIS HABILIDADES DE IA".into(),
                skill_gen,
            ),
        ])
    } else {
        Vec::new()
    }
}

fn scan_zen_spaces(container: &AppContainer, config: &Path) -> Vec<TuiRow> {
    if let Ok(profiles) = container.browser_adapter.scan_profiles(config) {
        let mut spaces = Vec::new();
        for prof in profiles {
            for space in prof.spaces {
                spaces.push(TuiItem {
                    id: space.id.clone(),
                    name: format!("{} ({})", space.name, prof.id.as_str()),
                    details: format!("{} abas ativas", space.open_tabs_count),
                    included: true,
                });
            }
        }
        TuiApp::group_into_rows(vec![(
            "spaces".into(),
            "WORKSPACES & ESPAÇOS DO NAVEGADOR".into(),
            spaces,
        )])
    } else {
        Vec::new()
    }
}

fn scan_ssh_keys(container: &AppContainer, home: &Path) -> Vec<TuiRow> {
    if let Ok(keys) = container.ssh_adapter.scan_ssh_keys(home) {
        let mut priv_keys = Vec::new();
        let mut pub_keys = Vec::new();
        let mut configs = Vec::new();

        for k in keys {
            let item = TuiItem {
                id: k.file_name.clone(),
                name: k.file_name.clone(),
                details: format!(
                    "{} • {:o} • {} bytes",
                    k.file_type.as_str(),
                    k.posix_mode,
                    k.size_bytes
                ),
                included: true,
            };
            match k.file_type {
                domain::entities::ssh_key::SshFileType::PrivateKey => priv_keys.push(item),
                domain::entities::ssh_key::SshFileType::PublicKey
                | domain::entities::ssh_key::SshFileType::Certificate => pub_keys.push(item),
                _ => configs.push(item),
            }
        }

        TuiApp::group_into_rows(vec![
            (
                "priv".into(),
                "CHAVES PRIVADAS (0600 - PROTEÇÃO MÁXIMA)".into(),
                priv_keys,
            ),
            (
                "pub".into(),
                "CHAVES PÚBLICAS & CERTIFICADOS (0644)".into(),
                pub_keys,
            ),
            (
                "conf".into(),
                "CONFIGURAÇÃO, KNOWN HOSTS & DEMAIS".into(),
                configs,
            ),
        ])
    } else {
        Vec::new()
    }
}

fn populate_snapshots(
    tui_app: &mut TuiApp,
    container: &AppContainer,
    home: &Path,
    backups_dir: &Path,
) {
    let mut scan_dirs = vec![backups_dir.to_path_buf()];
    let legacy_backups_dir = home.join("Antigravity/backups");
    if legacy_backups_dir != backups_dir && legacy_backups_dir.exists() {
        scan_dirs.push(legacy_backups_dir);
    }

    let mut seen_paths = std::collections::HashSet::new();
    for dir in scan_dirs {
        if dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                let mut paths: Vec<_> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_file() && p.to_string_lossy().ends_with(".tar.zst"))
                    .collect();
                paths.sort_by(|a, b| b.cmp(a));
                for p in paths {
                    if seen_paths.insert(p.clone()) {
                        let manifest = container.storage_adapter.read_manifest(&p).ok();
                        tui_app.add_snapshot_file(p, manifest);
                    }
                }
            }
        }
    }
}

fn execute_backup(
    container: &AppContainer,
    home: &Path,
    config: &Path,
    output_archive: PathBuf,
    filter: SelectionFilter,
    custom_dirs: Vec<CustomDirectoryEntry>,
    _dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        style("▶ Iniciando snapshot declarativo...").cyan().bold()
    );
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")?,
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message("Compactando perfis Zen, governança e pacotes...");

    let req = BackupRequest::new(config, home, &output_archive)
        .with_filter(filter)
        .with_custom_directories(custom_dirs);

    let report = container.backup_use_case.execute(req)?;
    pb.finish_and_clear();

    println!(
        "\n{}",
        style("✔ Snapshot gerado com sucesso!").green().bold()
    );
    println!(
        "  {} {}",
        style("ID do Snapshot:").bold(),
        report.snapshot_id
    );
    println!(
        "  {} {}",
        style("Arquivo:").bold(),
        report.output_path.display()
    );
    println!(
        "  {} {}",
        style("Checksum SHA-256:").bold(),
        report.checksum.as_str()
    );
    println!(
        "  {} {} arquivos",
        style("Total de Itens:").bold(),
        report.total_files
    );
    println!(
        "  {} {:.2} MB",
        style("Volume Estimado:").bold(),
        report.total_size_bytes as f64 / 1_048_576.0
    );
    println!(
        "  {} perfis Zen: {}",
        style("Zen Browser:").bold(),
        report.manifest.zen_profiles.len()
    );
    println!(
        "  {} skills: {}",
        style("Agentes IA:").bold(),
        report.manifest.agent_governance.skills.len()
    );
    println!(
        "  {} pacotes: {}",
        style("Pacotes Registrados:").bold(),
        report.manifest.packages.len()
    );
    println!(
        "  {} chaves/configs: {}",
        style("Chaves SSH:").bold(),
        report.manifest.ssh_keys.len()
    );

    Ok(())
}

struct RestoreExecutionParams<'a> {
    archive: &'a Path,
    config: &'a Path,
    home: &'a Path,
    target_custom: Option<PathBuf>,
    from_staging: Option<PathBuf>,
    install_packages: bool,
    dry_run: bool,
    selection_filter: Option<SelectionFilter>,
}

fn execute_restore(
    container: &AppContainer,
    params: RestoreExecutionParams<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        style("▶ Iniciando restauração do sistema...").cyan().bold()
    );

    // If package installation is requested and not dry run, validate or refresh sudo credentials cleanly
    if params.install_packages && !params.dry_run {
        println!(
            "{}",
            style("🔑 Privilégios de administrador necessários para instalar pacotes.").yellow().bold()
        );
        let sudo_status = std::process::Command::new("sudo")
            .arg("-v")
            .status();
        match sudo_status {
            Ok(status) if status.success() => {
                println!("{}", style("✔ Autenticação administrativa validada.").green());
            }
            _ => {
                eprintln!("{}", style("✖ Falha ou cancelamento na autenticação com sudo.").red().bold());
                return Err("Autenticação com sudo necessária para instalar pacotes.".into());
            }
        }
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")?,
    );
    pb.enable_steady_tick(Duration::from_millis(80));

    if params.from_staging.is_some() {
        pb.set_message("Restaurando a partir do staging em cache (modo instantâneo)...");
    } else {
        pb.set_message("Extraindo snapshot e normalizando navegador...");
    }

    let mut req = RestoreRequest::new(params.archive, params.config, params.home)
        .with_packages(params.install_packages, params.dry_run);

    if let Some(custom) = params.target_custom {
        req = req.with_custom_base(custom);
    }

    if let Some(staging) = params.from_staging {
        req = req.with_staging_override(staging);
    }

    if let Some(filter) = params.selection_filter {
        req = req.with_filter(filter);
    }

    let report = container.restore_use_case.execute(req)?;
    pb.finish_and_clear();

    println!("\n{}", style("✔ Restauração concluída!").green().bold());
    println!("  {} {}", style("Snapshot:").bold(), report.snapshot_id);
    println!(
        "  {} {} ➔ {}",
        style("Migração:").bold(),
        report.source_distro,
        report.target_distro
    );
    println!(
        "  {} {:?}",
        style("Perfis Zen Normalizados:").bold(),
        report.profiles_restored
    );
    println!(
        "  {} {}",
        style("Skills Restauradas:").bold(),
        report.skills_restored
    );

    if report.ssh_keys_restored > 0 {
        println!(
            "  {} {} instaladas com sucesso e permissões restritas (0700 ~/.ssh / 0600 chaves)",
            style("Chaves SSH:").bold(),
            report.ssh_keys_restored
        );
    }

    if report.git_configs_restored > 0 {
        println!(
            "  {} {} implantada(s) com sucesso no diretório do usuário",
            style("Configurações Globais Git:").bold(),
            report.git_configs_restored
        );
    }

    if !report.packages_installed.is_empty() {
        println!(
            "  {} {} instalados",
            style("Pacotes:").bold(),
            report.packages_installed.len()
        );
    }

    if !report.pending_packages.is_empty() {
        println!(
            "  {} {} pacotes nativos da distro anterior requerem atenção manual.",
            style("Aviso (Distro-Hop):").yellow().bold(),
            report.pending_packages.len()
        );
    }

    if report.fuse2_missing_warning {
        println!(
            "  {} libfuse2 não foi detectada. Instale libfuse2 para rodar AppImages.",
            style("Atenção AppImages:").yellow().bold()
        );
    }

    Ok(())
}

fn handle_inspect(
    container: &AppContainer,
    args: InspectArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = container.inspect_use_case.execute(&args.archive)?;

    if args.json {
        let json = serde_json::to_string_pretty(&summary)?;
        println!("{}", json);
    } else {
        println!("\n{}", style("=== Sumário do Snapshot ===").cyan().bold());
        println!("  {} {}", style("ID:").bold(), summary.snapshot_id);
        println!(
            "  {} {}",
            style("Criado em (UTC):").bold(),
            summary.created_at_utc
        );
        println!(
            "  {} {}",
            style("Host de Origem:").bold(),
            summary.source_host
        );
        println!(
            "  {} {}",
            style("Distro de Origem:").bold(),
            summary.source_distro
        );
        println!(
            "  {} {} ({} abas, {} spaces)",
            style("Zen Browser:").bold(),
            summary.zen_profiles_count,
            summary.total_tabs,
            summary.spaces_count
        );
        println!(
            "  {} {}",
            style("Pacotes Declarados:").bold(),
            summary.packages_count
        );
        println!(
            "  {} {}",
            style("Skills de IA:").bold(),
            summary.skills_count
        );
        println!(
            "  {} {}",
            style("Chaves SSH:").bold(),
            summary.ssh_keys_count
        );
        println!(
            "  {} {}",
            style("Pastas Customizadas:").bold(),
            summary.custom_dirs_count
        );
        if let Some(chk) = summary.checksum {
            println!("  {} {}", style("Checksum:").bold(), chk.as_str());
        }
    }

    Ok(())
}

fn handle_verify(
    container: &AppContainer,
    args: VerifyArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let report = container.verify_use_case.execute(&args.archive)?;

    println!(
        "\n{}",
        style("=== Verificação de Integridade ===").cyan().bold()
    );
    println!("  {} {}", style("Snapshot:").bold(), report.snapshot_id);
    println!(
        "  {} {}",
        style("Checksum Computado:").bold(),
        report.computed_checksum.as_str()
    );
    if let Some(ref decl) = report.declared_checksum {
        println!(
            "  {} {}",
            style("Checksum Declarado:").bold(),
            decl.as_str()
        );
    }
    println!(
        "  {} {}",
        style("Status:").bold(),
        style("Íntegro e Válido ✓").green().bold()
    );

    Ok(())
}

fn handle_slice(
    container: &AppContainer,
    args: SliceArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_archive = args.source;
    if !source_archive.exists() {
        return Err(format!(
            "Arquivo de snapshot de origem não encontrado: {:?}",
            source_archive
        )
        .into());
    }

    if args.interactive {
        let home = default_home();
        let config = home.join(".config/zen");
        handle_interactive(
            container,
            TuiMode::Restore,
            home,
            config,
            Vec::new(),
            None,
        )
    } else {
        let output_archive = args.output.unwrap_or_else(|| {
            let backups_dir = default_backups_dir();
            let _ = std::fs::create_dir_all(&backups_dir);
            let stem = source_archive
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "snapshot".to_string());
            let stem_clean = stem.strip_suffix(".tar.zst").unwrap_or(&stem);
            let timestamp = chrono_timestamp_slug();
            backups_dir.join(format!("{}-subset-{}.tar.zst", stem_clean, timestamp))
        });

        let mut filter = SelectionFilter::new();
        for p in &args.exclude_packages {
            if let Ok(name) = PackageName::new(p) {
                filter.toggle_package(name);
            }
        }
        for s in &args.exclude_skills {
            filter.toggle_skill(s.clone());
        }
        for k in &args.exclude_ssh_keys {
            filter.exclude_ssh_key(k.clone());
        }
        if args.no_ssh {
            if let Ok(manifest) = container.storage_adapter.read_manifest(&source_archive) {
                for k in manifest.ssh_keys {
                    filter.exclude_ssh_key(k.file_name);
                }
            }
        }
        if args.no_zen {
            filter.set_restore_zen(false);
        }
        if args.no_git_config {
            filter.set_restore_git_config(false);
        }
        for c in &args.exclude_custom_dirs {
            filter.toggle_custom_dir(c.clone());
        }

        execute_slice(
            container,
            &source_archive,
            output_archive,
            filter,
            args.staging_dir,
        )
    }
}

fn execute_slice(
    container: &AppContainer,
    source_archive: &Path,
    output_archive: PathBuf,
    filter: SelectionFilter,
    staging_dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        style("▶ Gerando sub-dump derivado (slice)...").cyan().bold()
    );
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")?,
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message("Filtrando e empacotando sub-dump em novo snapshot...");

    let mut req = SliceSnapshotRequest::new(source_archive, &output_archive, filter);
    if let Some(staging) = staging_dir {
        req = req.with_staging_dir(staging);
    }

    let report = container.slice_use_case.execute(req)?;
    pb.finish_and_clear();

    println!(
        "\n{}",
        style("✔ Sub-dump gerado com sucesso!").green().bold()
    );
    println!(
        "  {} {}",
        style("ID do Snapshot:").bold(),
        report.snapshot_id
    );
    println!(
        "  {} {}",
        style("Arquivo de Origem:").bold(),
        source_archive.display()
    );
    println!(
        "  {} {}",
        style("Novo Arquivo Gerado:").bold(),
        report.output_path.display()
    );
    println!(
        "  {} {}",
        style("Checksum SHA-256:").bold(),
        report.checksum.as_str()
    );
    println!(
        "  {} {} itens",
        style("Total de Arquivos:").bold(),
        report.total_files
    );
    println!(
        "  {} {:.2} MB",
        style("Volume Estimado:").bold(),
        report.total_size_bytes as f64 / 1_048_576.0
    );
    println!(
        "  {} perfis Zen: {}",
        style("Zen Browser:").bold(),
        report.manifest.zen_profiles.len()
    );
    println!(
        "  {} skills: {}",
        style("Agentes IA:").bold(),
        report.manifest.agent_governance.skills.len()
    );
    println!(
        "  {} pacotes: {}",
        style("Pacotes Registrados:").bold(),
        report.manifest.packages.len()
    );
    println!(
        "  {} chaves: {}",
        style("Chaves SSH:").bold(),
        report.manifest.ssh_keys.len()
    );
    println!(
        "  {} arquivos: {}",
        style("Configurações Git:").bold(),
        report.manifest.git_configs.len()
    );
    println!(
        "  {} pastas: {}",
        style("Pastas Customizadas:").bold(),
        report.manifest.custom_directories.len()
    );

    Ok(())
}

fn default_home() -> PathBuf {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if !sudo_user.is_empty() && sudo_user != "root" {
            let sudo_home = PathBuf::from(format!("/home/{}", sudo_user));
            if sudo_home.is_dir() {
                return sudo_home;
            }
        }
    }

    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/eduardorwll"))
}

fn default_backups_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| default_home())
        .join("backups")
}

fn chrono_timestamp_slug() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", now)
}
