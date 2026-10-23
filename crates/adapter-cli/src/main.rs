use std::path::PathBuf;
use std::time::Duration;
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use adapter_cli::args::{BackupArgs, Cli, Commands, InspectArgs, RestoreArgs, VerifyArgs};
use adapter_cli::di::AppContainer;
use adapter_cli::tui::{run_tui, TuiApp, TuiItem};
use adapter_storage::CustomDirsHandler;
use application::{BackupRequest, RestoreRequest};
use domain::entities::package::PackageScope;
use domain::entities::selection_filter::SelectionFilter;
use domain::ports::{AgentPort, BrowserPort};
use domain::value_objects::PackageName;

fn main() {
    let cli = Cli::parse();
    let container = AppContainer::bootstrap();

    let result = match cli.command {
        Some(Commands::Backup(args)) => handle_backup(&container, args),
        Some(Commands::Restore(args)) => handle_restore(&container, args),
        Some(Commands::Inspect(args)) => handle_inspect(&container, args),
        Some(Commands::Verify(args)) => handle_verify(&container, args),
        None => {
            // Default action: launch interactive TUI backup
            handle_backup(&container, BackupArgs {
                output: None,
                interactive: true,
                exclude_packages: Vec::new(),
                exclude_skills: Vec::new(),
                add_dirs: Vec::new(),
                dry_run: false,
                config_root: None,
                user_home: None,
            })
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("Erro:").red().bold(), e);
        std::process::exit(1);
    }
}

fn handle_backup(container: &AppContainer, args: BackupArgs) -> Result<(), Box<dyn std::error::Error>> {
    let home = args.user_home.unwrap_or_else(default_home);
    let config = args.config_root.unwrap_or_else(|| home.join(".config/zen"));
    let output_archive = args.output.unwrap_or_else(|| {
        let backups_dir = home.join("Antigravity/backups");
        let _ = std::fs::create_dir_all(&backups_dir);
        let timestamp = chrono_timestamp_slug();
        backups_dir.join(format!("distro-vault-{}.tar.zst", timestamp))
    });

    let (filter, custom_dirs) = if args.interactive {
        // Populate TuiApp with detected items
        let mut tui_app = TuiApp::new();

        // 1. Scan packages
        for port in &container.package_ports {
            if let Ok(pkgs) = port.query_installed() {
                for p in pkgs {
                    let item = TuiItem {
                        id: p.name.as_str().to_string(),
                        name: p.name.as_str().to_string(),
                        details: format!("{:?} • [{}] • {}", p.source, p.category.label(), p.version.as_deref().unwrap_or("latest")),
                        included: true,
                    };
                    if p.scope == PackageScope::User {
                        tui_app.user_packages.push(item);
                    } else {
                        tui_app.system_packages.push(item);
                    }
                }
            }
        }

        // 2. Scan skills
        if let Ok(gov) = container.agent_adapter.scan_governance(&home) {
            for s in gov.skills {
                tui_app.skills.push(TuiItem {
                    id: s.name.clone(),
                    name: s.name.clone(),
                    details: format!("{:?} • {}", s.category, s.description.as_deref().unwrap_or("No description")),
                    included: true,
                });
            }
        }

        // 3. Scan Zen spaces
        if let Ok(profiles) = container.browser_adapter.scan_profiles(&config) {
            for prof in profiles {
                for space in prof.spaces {
                    tui_app.zen_spaces.push(TuiItem {
                        id: space.id.clone(),
                        name: format!("{} ({})", space.name, prof.id.as_str()),
                        details: format!("{} abas abertas", space.open_tabs_count),
                        included: true,
                    });
                }
            }
        }

        // 4. Initial custom dirs from CLI flags
        for dir in &args.add_dirs {
            let _ = tui_app.add_custom_directory(&dir.to_string_lossy());
        }

        // Run TUI
        match run_tui(tui_app)? {
            Some(res) => res,
            None => {
                println!("{}", style("Operação cancelada pelo usuário na TUI.").yellow());
                return Ok(());
            }
        }
    } else {
        // Non-interactive CLI
        let mut filter = SelectionFilter::new();
        for p in &args.exclude_packages {
            if let Ok(name) = PackageName::new(p) {
                filter.toggle_package(name);
            }
        }
        for s in &args.exclude_skills {
            filter.toggle_skill(s.clone());
        }

        let mut custom_dirs = Vec::new();
        for d in &args.add_dirs {
            if let Ok(entry) = CustomDirsHandler::inspect_directory(d, None) {
                custom_dirs.push(entry);
            }
        }

        (filter, custom_dirs)
    };

    println!("{}", style("▶ Iniciando snapshot declarativo...").cyan().bold());
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")?,
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message("Compactando perfis Zen, governança e pacotes...");

    let req = BackupRequest::new(&config, &home, &output_archive)
        .with_filter(filter)
        .with_custom_directories(custom_dirs);

    let report = container.backup_use_case.execute(req)?;
    pb.finish_and_clear();

    println!("\n{}", style("✔ Snapshot gerado com sucesso!").green().bold());
    println!("  {} {}", style("ID do Snapshot:").bold(), report.snapshot_id);
    println!("  {} {}", style("Arquivo:").bold(), report.output_path.display());
    println!("  {} {}", style("Checksum SHA-256:").bold(), report.checksum.as_str());
    println!("  {} {} arquivos", style("Total de Itens:").bold(), report.total_files);
    println!("  {} {:.2} MB", style("Volume Estimado:").bold(), report.total_size_bytes as f64 / 1_048_576.0);
    println!("  {} perfis Zen: {}", style("Zen Browser:").bold(), report.manifest.zen_profiles.len());
    println!("  {} skills: {}", style("Agentes IA:").bold(), report.manifest.agent_governance.skills.len());
    println!("  {} pacotes: {}", style("Pacotes Registrados:").bold(), report.manifest.packages.len());

    Ok(())
}

fn handle_restore(container: &AppContainer, args: RestoreArgs) -> Result<(), Box<dyn std::error::Error>> {
    let home = args.target_home.unwrap_or_else(default_home);
    let config = args.target_config.unwrap_or_else(|| home.join(".config/zen"));

    println!("{}", style("▶ Iniciando restauração do sistema...").cyan().bold());
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")?,
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message("Extraindo snapshot e normalizando navegador...");

    let mut req = RestoreRequest::new(&args.archive, &config, &home)
        .with_packages(args.install_packages, args.dry_run);

    if let Some(custom) = args.target_custom {
        req = req.with_custom_base(custom);
    }

    let report = container.restore_use_case.execute(req)?;
    pb.finish_and_clear();

    println!("\n{}", style("✔ Restauração concluída!").green().bold());
    println!("  {} {}", style("Snapshot:").bold(), report.snapshot_id);
    println!("  {} {} ➔ {}", style("Migração:").bold(), report.source_distro, report.target_distro);
    println!("  {} {:?}", style("Perfis Zen Normalizados:").bold(), report.profiles_restored);
    println!("  {} {}", style("Skills Restauradas:").bold(), report.skills_restored);

    if !report.packages_installed.is_empty() {
        println!("  {} {} instalados", style("Pacotes:").bold(), report.packages_installed.len());
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

fn handle_inspect(container: &AppContainer, args: InspectArgs) -> Result<(), Box<dyn std::error::Error>> {
    let summary = container.inspect_use_case.execute(&args.archive)?;

    if args.json {
        let json = serde_json::to_string_pretty(&summary)?;
        println!("{}", json);
    } else {
        println!("\n{}", style("=== Sumário do Snapshot ===").cyan().bold());
        println!("  {} {}", style("ID:").bold(), summary.snapshot_id);
        println!("  {} {}", style("Criado em (UTC):").bold(), summary.created_at_utc);
        println!("  {} {}", style("Host de Origem:").bold(), summary.source_host);
        println!("  {} {}", style("Distro de Origem:").bold(), summary.source_distro);
        println!("  {} {} ({} abas, {} spaces)", style("Zen Browser:").bold(), summary.zen_profiles_count, summary.total_tabs, summary.spaces_count);
        println!("  {} {}", style("Pacotes Declarados:").bold(), summary.packages_count);
        println!("  {} {}", style("Skills de IA:").bold(), summary.skills_count);
        println!("  {} {}", style("Pastas Customizadas:").bold(), summary.custom_dirs_count);
        if let Some(chk) = summary.checksum {
            println!("  {} {}", style("Checksum:").bold(), chk.as_str());
        }
    }

    Ok(())
}

fn handle_verify(container: &AppContainer, args: VerifyArgs) -> Result<(), Box<dyn std::error::Error>> {
    let report = container.verify_use_case.execute(&args.archive)?;

    println!("\n{}", style("=== Verificação de Integridade ===").cyan().bold());
    println!("  {} {}", style("Snapshot:").bold(), report.snapshot_id);
    println!("  {} {}", style("Checksum Computado:").bold(), report.computed_checksum.as_str());
    if let Some(ref decl) = report.declared_checksum {
        println!("  {} {}", style("Checksum Declarado:").bold(), decl.as_str());
    }
    println!("  {} {}", style("Status:").bold(), style("Íntegro e Válido ✓").green().bold());

    Ok(())
}

fn default_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/eduardorwll"))
}

fn chrono_timestamp_slug() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    format!("{}", now)
}
