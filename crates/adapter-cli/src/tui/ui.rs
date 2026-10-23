use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

use super::app::TuiApp;

pub fn draw(f: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(8),    // List area
            Constraint::Length(3), // Status & Keybindings Footer
        ])
        .split(f.area());

    // 1. Header
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(13, 17, 23)));
    let header_text = Paragraph::new(Line::from(vec![
        Span::styled("distro-vault ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(":: Snapshot Interativo & Gestão de Exclusões", Style::default().fg(Color::White)),
    ]))
    .alignment(Alignment::Center)
    .block(title_block);
    f.render_widget(header_text, chunks[0]);

    // 2. Tabs
    let tab_titles = vec![
        format!("[1] Apps & Usuário ({})", app.user_packages.len()),
        format!("[2] Sistema Low-Level ({})", app.system_packages.len()),
        format!("[3] Skills ({})", app.skills.len()),
        format!("[4] AppImages ({})", app.appimages.len()),
        format!("[5] Pastas Extras ({})", app.custom_dirs.len()),
        format!("[6] Zen Spaces ({})", app.zen_spaces.len()),
    ];

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(" Menus "))
        .select(app.current_tab)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::Rgb(22, 27, 34)),
        );
    f.render_widget(tabs, chunks[1]);

    // 3. Main List Content
    let current_cursor = app.cursors[app.current_tab];
    let (list_items, title) = match app.current_tab {
        0 => {
            let items: Vec<ListItem> = app
                .user_packages
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let is_selected = i == current_cursor;
                    render_item(&item.name, &item.details, item.included, is_selected)
                })
                .collect();
            (items, " 📦 Pacotes Instalados pelo Usuário (Apps, Ferramentas CLI, Dev & Flatpaks) ")
        }
        1 => {
            let items: Vec<ListItem> = app
                .system_packages
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let is_selected = i == current_cursor;
                    render_item(&item.name, &item.details, item.included, is_selected)
                })
                .collect();
            (items, " ⚙️ Pacotes de Sistema Low-Level (Kernel, Drivers, Base, Filesystems & Daemons) ")
        }
        2 => {
            let items: Vec<ListItem> = app
                .skills
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let is_selected = i == current_cursor;
                    render_item(&item.name, &item.details, item.included, is_selected)
                })
                .collect();
            (items, " 🤖 Skills de IA & Governança (~/.agents/skills) ")
        }
        3 => {
            let items: Vec<ListItem> = app
                .appimages
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let is_selected = i == current_cursor;
                    render_item(&item.name, &item.details, item.included, is_selected)
                })
                .collect();
            (items, " 🚀 Aplicativos AppImage & Gear Lever ")
        }
        4 => {
            let items: Vec<ListItem> = app
                .custom_dirs
                .iter()
                .enumerate()
                .map(|(i, dir)| {
                    let is_selected = i == current_cursor;
                    let label = format!("{} (custom/{})", dir.label, dir.id);
                    let details = format!("{} arquivos • {:.2} MB", dir.file_count, dir.size_bytes as f64 / 1_048_576.0);
                    render_item(&label, &details, true, is_selected)
                })
                .collect();
            (items, " 📁 Pastas Extras do Usuário (Pressione 'A' para adicionar) ")
        }
        5 => {
            let items: Vec<ListItem> = app
                .zen_spaces
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let is_selected = i == current_cursor;
                    render_item(&item.name, &item.details, item.included, is_selected)
                })
                .collect();
            (items, " 🌐 Zen Browser Spaces & Workspaces ")
        }
        _ => (Vec::new(), ""),
    };

    let list_widget = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(list_widget, chunks[2]);

    // 4. Footer & Keybindings
    let status = if let Some(ref msg) = app.status_message {
        Span::styled(format!(" {} ", msg), Style::default().fg(Color::Yellow))
    } else {
        Span::styled(
            " [Tab] Próxima Aba  [↑/↓] Navegar  [Espaço] Alternar  [A] Adicionar Pasta  [Enter/F10] Gerar Snapshot  [Q] Sair ",
            Style::default().fg(Color::DarkGray),
        )
    };

    let footer_text = Paragraph::new(Line::from(vec![status]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer_text, chunks[3]);

    // 5. Input Popup for Custom Directory
    if app.input_mode {
        render_input_popup(f, &app.input_buffer);
    }
}

fn render_item(name: &str, details: &str, included: bool, is_selected: bool) -> ListItem<'static> {
    let (mark, mark_style) = if included {
        ("[x] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        ("[ ] ", Style::default().fg(Color::Red))
    };

    let bg = if is_selected {
        Color::Rgb(33, 38, 45)
    } else {
        Color::Reset
    };

    let prefix = if is_selected { "▶ " } else { "  " };

    let line = Line::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(Color::Cyan)),
        Span::styled(mark.to_string(), mark_style),
        Span::styled(name.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  — {}", details), Style::default().fg(Color::Gray)),
    ]);

    ListItem::new(line).style(Style::default().bg(bg))
}

fn render_input_popup(f: &mut Frame, buffer: &str) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Adicionar Diretório Customizado ")
        .style(Style::default().bg(Color::Rgb(22, 27, 34)).fg(Color::Cyan));

    let content = vec![
        Line::from(vec![Span::raw("Digite o caminho completo (ex: /home/usuario/Documentos):")]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Yellow)),
            Span::styled(buffer, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("_", Style::default().fg(Color::Cyan).add_modifier(Modifier::SLOW_BLINK)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "[Enter] Confirmar  •  [Esc] Cancelar",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
