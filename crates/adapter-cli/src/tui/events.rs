use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::TuiApp;

pub fn handle_key_event(app: &mut TuiApp, key: KeyEvent) {
    if app.input_mode {
        match key.code {
            KeyCode::Enter => {
                let buffer = app.input_buffer.clone();
                if !buffer.trim().is_empty() {
                    match app.add_custom_directory(&buffer) {
                        Ok(_) => {
                            app.input_buffer.clear();
                            app.input_mode = false;
                        }
                        Err(e) => {
                            app.status_message = Some(format!("Erro: {}", e));
                        }
                    }
                } else {
                    app.input_mode = false;
                }
            }
            KeyCode::Esc => {
                app.input_buffer.clear();
                app.input_mode = false;
                app.status_message = Some("Adição cancelada".to_string());
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

    match key.code {
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
    }
}
