use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::Duration;

use domain::entities::custom_directory::CustomDirectoryEntry;
use domain::entities::selection_filter::SelectionFilter;

use super::app::TuiApp;
use super::events::handle_key_event;
use super::ui::draw;

/// Actions returned by the interactive TUI event loop
#[derive(Debug, Clone)]
pub enum TuiAction {
    Backup {
        filter: Box<SelectionFilter>,
        custom_dirs: Vec<CustomDirectoryEntry>,
    },
    Restore {
        archive_path: PathBuf,
        from_staging: Option<PathBuf>,
        install_packages: bool,
        dry_run: bool,
        selection_filter: Option<Box<SelectionFilter>>,
    },
    Slice {
        source_archive: PathBuf,
        output_archive: PathBuf,
        selection_filter: Box<SelectionFilter>,
    },
}

/// RAII Guard that guarantees terminal raw mode is cleanly restored on drop
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

/// Runs the interactive TUI event loop, returning the selected TuiAction if confirmed
pub fn run_tui(mut app: TuiApp) -> io::Result<Option<TuiAction>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(&mut app, key);
            }
        }

        if app.should_start_backup {
            let filter = app.build_selection_filter();
            return Ok(Some(TuiAction::Backup {
                filter: Box::new(filter),
                custom_dirs: app.custom_dirs,
            }));
        }

        if app.should_start_restore {
            if let Some(snap) = app.selected_snapshot() {
                let from_staging = app.detect_staging_for_selected();
                let selection_filter = Some(Box::new(app.restore_tree.build_selection_filter()));
                return Ok(Some(TuiAction::Restore {
                    archive_path: snap.path.clone(),
                    from_staging,
                    install_packages: app.restore_install_packages,
                    dry_run: app.restore_dry_run,
                    selection_filter,
                }));
            } else {
                app.status_message = Some("Nenhum snapshot selecionado para restaurar.".into());
                app.should_start_restore = false;
            }
        }

        if app.should_start_slice {
            if let Some(snap) = app.selected_snapshot() {
                let output_archive = match app.get_slice_default_output() {
                    Some(out) => out,
                    None => {
                        app.status_message =
                            Some("Erro ao calcular caminho de saída do sub-dump.".into());
                        app.should_start_slice = false;
                        continue;
                    }
                };
                let selection_filter = Box::new(app.restore_tree.build_selection_filter());
                return Ok(Some(TuiAction::Slice {
                    source_archive: snap.path.clone(),
                    output_archive,
                    selection_filter,
                }));
            } else {
                app.status_message =
                    Some("Nenhum snapshot selecionado para gerar sub-dump.".into());
                app.should_start_slice = false;
            }
        }

        if app.should_quit {
            return Ok(None);
        }
    }
}
