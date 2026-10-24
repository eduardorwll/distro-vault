use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use super::app::{InputTarget, RestoreFocus, TuiApp, TuiMode};
use super::explorer::{ExplorerSelection, ExplorerTarget};

pub fn handle_key_event(app: &mut TuiApp, key: KeyEvent) {
    // 1. Interactive File & Directory Explorer handling
    if app.file_explorer.is_open {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.file_explorer.prev();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.file_explorer.next();
            }
            KeyCode::Backspace => {
                app.file_explorer.go_up();
            }
            KeyCode::Enter => {
                let target = app.file_explorer.target;
                if let Some(sel) = app.file_explorer.enter_or_select() {
                    apply_explorer_selection(app, target, sel);
                }
            }
            KeyCode::Char(' ') | KeyCode::Char('s') | KeyCode::Char('S') => {
                let target = app.file_explorer.target;
                if let Some(sel) = app.file_explorer.confirm_selection() {
                    apply_explorer_selection(app, target, sel);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                app.file_explorer.close();
                app.status_message = Some("Navegação cancelada.".to_string());
            }
            _ => {}
        }
        return;
    }

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
                                app.status_message =
                                    Some("Arquivo não encontrado no caminho informado.".into());
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
                let start = std::env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("/"));
                app.file_explorer.open(&start, ExplorerTarget::CustomDir);
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
            KeyCode::Tab | KeyCode::BackTab => {
                app.toggle_restore_focus();
            }
            KeyCode::Down | KeyCode::Char('j') => match app.restore_focus {
                RestoreFocus::SnapshotsList => app.next_snapshot(),
                RestoreFocus::ItemTree => app.restore_tree.next(),
            },
            KeyCode::Up | KeyCode::Char('k') => match app.restore_focus {
                RestoreFocus::SnapshotsList => app.prev_snapshot(),
                RestoreFocus::ItemTree => app.restore_tree.prev(),
            },
            KeyCode::Enter => match app.restore_focus {
                RestoreFocus::SnapshotsList => {
                    app.restore_focus = RestoreFocus::ItemTree;
                }
                RestoreFocus::ItemTree => {
                    app.restore_tree.toggle_expanded();
                }
            },
            KeyCode::Right | KeyCode::Left | KeyCode::Char('e') | KeyCode::Char('E') => {
                if app.restore_focus == RestoreFocus::ItemTree {
                    app.restore_tree.toggle_expanded();
                }
            }
            KeyCode::Char(' ') | KeyCode::Char('x') | KeyCode::Char('X') => {
                if app.restore_focus == RestoreFocus::ItemTree {
                    app.restore_tree.toggle_selected();
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                app.toggle_restore_install_packages();
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                app.toggle_restore_dry_run();
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                let start = app.default_backups_dir.clone();
                app.file_explorer.open(&start, ExplorerTarget::SnapshotOrFolder);
            }
            KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::F(10) => {
                app.should_start_restore = true;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                if app.restore_focus == RestoreFocus::ItemTree {
                    app.restore_focus = RestoreFocus::SnapshotsList;
                } else {
                    app.should_quit = true;
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }
            _ => {}
        },
    }
}

fn apply_explorer_selection(app: &mut TuiApp, target: ExplorerTarget, sel: ExplorerSelection) {
    match target {
        ExplorerTarget::CustomDir => match sel {
            ExplorerSelection::Directory(path) => {
                match app.add_custom_directory(&path.to_string_lossy()) {
                    Ok(_) => {
                        app.status_message = Some(format!("Pasta adicionada para backup: {}", path.display()));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("Erro ao adicionar pasta: {}", e));
                    }
                }
            }
            ExplorerSelection::SnapshotFile(_) => {}
        },
        ExplorerTarget::SnapshotOrFolder => match sel {
            ExplorerSelection::SnapshotFile(path) => {
                app.add_snapshot_file(path.clone(), None);
                app.status_message = Some(format!("Snapshot adicionado: {}", path.display()));
            }
            ExplorerSelection::Directory(path) => {
                let count = app.scan_and_add_backup_folder(&path);
                app.status_message = Some(format!(
                    "Pasta de backups selecionada: {} ({} snapshots encontrados)",
                    path.display(),
                    count
                ));
            }
        },
    }
}
