use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

use super::app::{TuiApp, TuiRow};

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
        format!("[1] Apps & Usuário ({})", app.count_items(0)),
        format!("[2] Sistema Low-Level ({})", app.count_items(1)),
        format!("[3] Dependências ({})", app.count_items(2)),
        format!("[4] Skills ({})", app.count_items(3)),
        format!("[5] AppImages ({})", app.count_items(4)),
        format!("[6] Pastas Extras ({})", app.custom_dirs.len()),
        format!("[7] Zen Spaces ({})", app.count_items(6)),
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
        0 => (
            render_rows(&app.user_packages, current_cursor),
            " 📦 Pacotes Instalados pelo Usuário (Categorizados por Função) ",
        ),
        1 => (
            render_rows(&app.system_packages, current_cursor),
            " ⚙️ Pacotes de Sistema Low-Level (Kernel, Drivers, Base, Filesystems) ",
        ),
        2 => (
            render_rows(&app.dependency_packages, current_cursor),
            " 🔗 Dependências do Sistema (Instaladas Indiretamente / pacman -Qd) ",
        ),
        3 => (
            render_rows(&app.skills, current_cursor),
            " 🤖 Skills de IA & Governança (~/.agents/skills) ",
        ),
        4 => (
            render_rows(&app.appimages, current_cursor),
            " 🚀 Aplicativos AppImage & Gear Lever ",
        ),
        5 => {
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
        6 => (
            render_rows(&app.zen_spaces, current_cursor),
            " 🌐 Zen Browser Spaces & Workspaces ",
        ),
        _ => (Vec::new(), ""),
    };

    let list_widget = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(title));

    let state = &mut app.list_states[app.current_tab];
    state.select(Some(current_cursor));
    f.render_stateful_widget(list_widget, chunks[2], state);

    // 4. Footer & Keybindings
    let status = if let Some(ref msg) = app.status_message {
        Span::styled(format!(" {} ", msg), Style::default().fg(Color::Yellow))
    } else {
        Span::styled(
            " [Tab] Próxima Aba  [↑/↓] Navegar  [Espaço] Alternar Item/Categoria  [A] Adicionar Pasta  [Enter/F10] Gerar Snapshot  [Q] Sair ",
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

fn render_rows(rows: &[TuiRow], current_cursor: usize) -> Vec<ListItem<'static>> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected = i == current_cursor;
            match row {
                TuiRow::Header { title, count, .. } => render_header(title, *count, is_selected),
                TuiRow::Item { item, .. } => render_item(&item.name, &item.details, item.included, is_selected),
            }
        })
        .collect()
}

fn render_header(title: &str, count: usize, is_selected: bool) -> ListItem<'static> {
    let (prefix, text_color, bg_color) = if is_selected {
        ("▶ ", Color::Yellow, Color::Rgb(28, 38, 54))
    } else {
        ("  ", Color::Cyan, Color::Rgb(18, 24, 32))
    };

    let header_label = format!("── [ {} ] ({}) ", title, count);
    let fill = "─".repeat(50usize.saturating_sub(header_label.len()));

    let line = Line::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(header_label, Style::default().fg(text_color).add_modifier(Modifier::BOLD)),
        Span::styled(fill, Style::default().fg(Color::DarkGray)),
    ]);

    ListItem::new(line).style(Style::default().bg(bg_color))
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
