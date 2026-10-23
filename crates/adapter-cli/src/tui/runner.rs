use std::io::{self, stdout};
use std::time::Duration;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use domain::entities::custom_directory::CustomDirectoryEntry;
use domain::entities::selection_filter::SelectionFilter;

use super::app::TuiApp;
use super::events::handle_key_event;
use super::ui::draw;

/// RAII Guard that guarantees terminal raw mode is cleanly restored on drop
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

/// Runs the interactive TUI event loop, returning the computed SelectionFilter and Custom Dirs if confirmed
pub fn run_tui(mut app: TuiApp) -> io::Result<Option<(SelectionFilter, Vec<CustomDirectoryEntry>)>> {
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
            return Ok(Some((filter, app.custom_dirs)));
        }

        if app.should_quit {
            return Ok(None);
        }
    }
}
