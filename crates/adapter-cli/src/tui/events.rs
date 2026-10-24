use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{InputTarget, TuiApp, TuiMode};

pub fn handle_key_event(app: &mut TuiApp, key: KeyEvent) {
    if app.input_mode {
        match key.code {
            KeyCode::Enter => {
                let buffer = app.input_buffer.clone();
                if !buffer.trim().is_empty() {
                    match app.input_target {
                        InputTarget::CustomDir => match app.add_custom_directory(&buffer) {
                            Ok(_) => {
                                app.input_buffer.clear();
                                app.input_mode = false;
                            }
                            Err(e) => {
                                app.status_message = Some(format!("Erro: {}", e));
                            }
                        },
                        InputTarget::SnapshotPath => {
                            let path = PathBuf::from(buffer.trim());
                            if path.exists() {
                                app.add_snapshot_file(path, None);
                                app.status_message = Some("Snapshot adicionado à lista.".into());
                                app.input_buffer.clear();
                                app.input_mode = false;
                            } else {
                                app.status_message = Some("Arquivo não encontrado no caminho informado.".into());
                            }
                        }
                    }
                } else {
                    app.input_mode = false;
                }
            }
            KeyCode::Esc => {
                app.input_buffer.clear();
                app.input_mode = false;
                app.status_message = Some("Ação cancelada".to_string());
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            _ => {}
        }
        return;
    }

    // Global Mode Toggle
    match key.code {
        KeyCode::F(2) => {
            app.set_mode(TuiMode::Backup);
            return;
        }
        KeyCode::F(3) => {
            app.set_mode(TuiMode::Restore);
            return;
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.toggle_mode();
            return;
        }
        _ => {}
    }

    match app.mode {
        TuiMode::Backup => match key.code {
            KeyCode::Tab => {
                app.next_tab();
            }
            KeyCode::BackTab => {
                app.prev_tab();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_item();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.prev_item();
            }
            KeyCode::Char(' ') => {
                app.toggle_selected();
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                app.input_target = InputTarget::CustomDir;
                app.input_mode = true;
                app.input_buffer.clear();
            }
            KeyCode::Enter | KeyCode::F(10) => {
                app.should_start_backup = true;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                app.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }
            _ => {}
        },
        TuiMode::Restore => match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_snapshot();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.prev_snapshot();
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                app.toggle_restore_install_packages();
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                app.toggle_restore_dry_run();
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                app.input_target = InputTarget::SnapshotPath;
                app.input_mode = true;
                app.input_buffer.clear();
            }
            KeyCode::Enter | KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::F(10) => {
                app.should_start_restore = true;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                app.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }
            _ => {}
        },
    }
}
