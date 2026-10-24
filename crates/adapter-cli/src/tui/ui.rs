use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

use super::app::{InputTarget, TuiApp, TuiMode, TuiRow};

pub fn draw(f: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs / Sub-header
            Constraint::Min(8),    // Main area
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // 1. Header with Mode Indicator
    let (backup_style, restore_style) = match app.mode {
        TuiMode::Backup => (
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            Style::default().fg(Color::DarkGray),
        ),
        TuiMode::Restore => (
            Style::default().fg(Color::DarkGray),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    };

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(13, 17, 23)));
    let header_text = Paragraph::new(Line::from(vec![
        Span::styled("distro-vault ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(":: ", Style::default().fg(Color::DarkGray)),
        Span::styled("[F2] 📦 NOVO BACKUP", backup_style),
        Span::styled("   •   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[F3] 🔄 RECUPERAÇÃO (RESTORE)", restore_style),
    ]))
    .alignment(Alignment::Center)
    .block(title_block);
    f.render_widget(header_text, chunks[0]);

    match app.mode {
        TuiMode::Backup => draw_backup_mode(f, app, chunks[1], chunks[2]),
        TuiMode::Restore => draw_restore_mode(f, app, chunks[1], chunks[2]),
    }

    // 4. Footer & Keybindings
    let status = if let Some(ref msg) = app.status_message {
        Span::styled(format!(" {} ", msg), Style::default().fg(Color::Yellow))
    } else {
        match app.mode {
            TuiMode::Backup => Span::styled(
                " [Tab] Alternar Aba  [↑/↓] Navegar  [Espaço] Alternar Item/Grupo  [A] Add Pasta  [Enter] Snapshot  [F3] Modo Recuperar  [Q] Sair ",
                Style::default().fg(Color::DarkGray),
            ),
            TuiMode::Restore => Span::styled(
                " [↑/↓] Selecionar Snapshot  [P] Alternar Pacotes  [D] Alternar Dry-Run  [A] Outro Arquivo  [Enter/R] RESTAURAR  [F2] Modo Backup  [Q] Sair ",
                Style::default().fg(Color::DarkGray),
            ),
        }
    };

    let footer_text = Paragraph::new(Line::from(vec![status]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer_text, chunks[3]);

    // 5. Input Popup (Custom Directory or Snapshot Path)
    if app.input_mode {
        render_input_popup(f, &app.input_buffer, app.input_target);
    }
}

fn draw_backup_mode(f: &mut Frame, app: &mut TuiApp, tab_area: Rect, main_area: Rect) {
    let tab_titles = vec![
        format!("[1] Apps & Usuário ({}/{})", app.count_selected_items(0), app.count_items(0)),
        format!("[2] Sistema Low-Level ({}/{})", app.count_selected_items(1), app.count_items(1)),
        format!("[3] Dependências ({}/{})", app.count_selected_items(2), app.count_items(2)),
        format!("[4] Skills ({}/{})", app.count_selected_items(3), app.count_items(3)),
        format!("[5] AppImages ({}/{})", app.count_selected_items(4), app.count_items(4)),
        format!("[6] Pastas Extras ({})", app.custom_dirs.len()),
        format!("[7] Zen Spaces ({}/{})", app.count_selected_items(6), app.count_items(6)),
    ];

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(" Menus de Seleção do Snapshot "))
        .select(app.current_tab)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::Rgb(22, 27, 34)),
        );
    f.render_widget(tabs, tab_area);

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
    f.render_stateful_widget(list_widget, main_area, state);
}

fn draw_restore_mode(f: &mut Frame, app: &mut TuiApp, sub_area: Rect, main_area: Rect) {
    let pkg_label = if app.restore_install_packages { "SIM" } else { "NÃO" };
    let dry_label = if app.restore_dry_run { "ATIVADO (Simulação)" } else { "DESATIVADO" };

    let banner = Paragraph::new(Line::from(vec![
        Span::styled(" Diretório Padrão: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("~/Antigravity/backups/  •  ", Style::default().fg(Color::White)),
        Span::styled("Snapshots Encontrados: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}  •  ", app.snapshots.len()), Style::default().fg(Color::White)),
        Span::styled("[P] Instalar Pacotes: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}  •  ", pkg_label), Style::default().fg(Color::White)),
        Span::styled("[D] Dry-Run: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled(dry_label, Style::default().fg(Color::White)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).title(" Painel de Controle de Recuperação "));
    f.render_widget(banner, sub_area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(main_area);

    // Left Column: List of Snapshots
    let list_items: Vec<ListItem> = if app.snapshots.is_empty() {
        vec![
            ListItem::new(Line::from(vec![
                Span::styled("  Nenhum snapshot encontrado em ~/Antigravity/backups/", Style::default().fg(Color::Yellow)),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Pressione 'A' para abrir um arquivo .tar.zst manualmente.", Style::default().fg(Color::DarkGray)),
            ])),
        ]
    } else {
        app.snapshots
            .iter()
            .enumerate()
            .map(|(i, snap)| {
                let is_selected = i == app.snapshot_cursor;
                let prefix = if is_selected { "▶ " } else { "  " };
                let size_mb = snap.size_bytes as f64 / 1_048_576.0;
                let line = Line::from(vec![
                    Span::styled(prefix.to_string(), Style::default().fg(Color::Green)),
                    Span::styled("📦 ", Style::default().fg(Color::Yellow)),
                    Span::styled(snap.filename.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("  ({:.2} MB)", size_mb), Style::default().fg(Color::Gray)),
                ]);
                let bg = if is_selected { Color::Rgb(28, 42, 34) } else { Color::Reset };
                ListItem::new(line).style(Style::default().bg(bg))
            })
            .collect()
    };

    let list_widget = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(" 📂 Snapshots Disponíveis "));

    if !app.snapshots.is_empty() {
        app.snapshot_list_state.select(Some(app.snapshot_cursor));
    }
    f.render_stateful_widget(list_widget, cols[0], &mut app.snapshot_list_state);

    // Right Column: Details & Preview
    let details_widget = if let Some(snap) = app.selected_snapshot() {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("Arquivo: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(snap.path.display().to_string(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Tamanho: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.2} MB ({} bytes)", snap.size_bytes as f64 / 1_048_576.0, snap.size_bytes), Style::default().fg(Color::White)),
        ]));

        if let Some(ref m) = snap.manifest {
            lines.push(Line::from(vec![
                Span::styled("Snapshot ID: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(&m.snapshot_id, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Criado em: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(&m.created_at_utc, Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Host Origem: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(&m.source_host, Style::default().fg(Color::White)),
                Span::styled("  •  Distro Origem: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}", m.source_distro), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("── CONTEÚDO CATALOGADO ──", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            ]));
            let total_tabs: usize = m.zen_profiles.iter().map(|p| p.total_tabs()).sum();
            lines.push(Line::from(vec![
                Span::styled("  🌐 Zen Browser: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} perfis, {} abas ativas", m.zen_profiles.len(), total_tabs), Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  🤖 Agentes IA: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} skills registradas (~/.agents/skills)", m.agent_governance.skills.len()), Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  📦 Pacotes: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} pacotes no catálogo declarativo", m.packages.len()), Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  📁 Pastas Extras: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} diretórios customizados arquivados", m.custom_directories.len()), Style::default().fg(Color::White)),
            ]));
            if let Some(ref cs) = m.checksum {
                lines.push(Line::from(vec![
                    Span::styled("  🔒 SHA-256: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(cs.as_str(), Style::default().fg(Color::DarkGray)),
                ]));
            }
        } else {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("⚠️  Manifesto não pôde ser pré-visualizado para este arquivo.", Style::default().fg(Color::Yellow)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("── OPÇÕES DE RESTAURAÇÃO ──", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        ]));
        let pkg_indicator = if app.restore_install_packages {
            Span::styled(" [x] ATIVADO (reinstala pacotes via gerenciador)", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" [ ] DESATIVADO (restaura apenas configs/navegador/skills)", Style::default().fg(Color::Red))
        };
        lines.push(Line::from(vec![
            Span::styled("  [P] Instalação de Pacotes: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            pkg_indicator,
        ]));

        let dry_indicator = if app.restore_dry_run {
            Span::styled(" [x] ATIVADO (simula sem alterar o sistema)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" [ ] DESATIVADO (restauração real nos discos)", Style::default().fg(Color::DarkGray))
        };
        lines.push(Line::from(vec![
            Span::styled("  [D] Modo Simulação (Dry-Run): ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            dry_indicator,
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("👉 Pressione [Enter] ou [R] para restaurar este snapshot agora.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));

        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" 🔍 Inspeção & Detalhes do Snapshot "))
    } else {
        Paragraph::new(vec![
            Line::from(""),
            Line::from("  Nenhum snapshot selecionado."),
            Line::from("  Use as setas [↑/↓] para navegar ou pressione 'A' para adicionar um arquivo .tar.zst."),
        ])
        .block(Block::default().borders(Borders::ALL).title(" 🔍 Inspeção "))
    };

    f.render_widget(details_widget, cols[1]);
}

fn render_rows(rows: &[TuiRow], current_cursor: usize) -> Vec<ListItem<'static>> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected = i == current_cursor;
            match row {
                TuiRow::Header { category_id, title, count } => {
                    let selected_count = rows
                        .iter()
                        .filter(|r| match r {
                            TuiRow::Item { category_id: c, item } => c == category_id && item.included,
                            _ => false,
                        })
                        .count();
                    render_header(title, selected_count, *count, is_selected)
                }
                TuiRow::Item { item, .. } => render_item(&item.name, &item.details, item.included, is_selected),
            }
        })
        .collect()
}

fn render_header(title: &str, selected: usize, total: usize, is_selected: bool) -> ListItem<'static> {
    let (prefix, text_color, bg_color) = if is_selected {
        ("▶ ", Color::Yellow, Color::Rgb(28, 38, 54))
    } else {
        ("  ", Color::Cyan, Color::Rgb(18, 24, 32))
    };

    let check = if total == 0 || selected == 0 {
        "[ ] "
    } else if selected == total {
        "[x] "
    } else {
        "[-] "
    };

    let header_label = format!("{}[ {} ] ({}/{}) ", check, title, selected, total);
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

fn render_input_popup(f: &mut Frame, buffer: &str, target: InputTarget) {
    let area = centered_rect(65, 22, f.area());
    f.render_widget(Clear, area);

    let (title, prompt_desc) = match target {
        InputTarget::CustomDir => (
            " Adicionar Diretório Customizado ",
            "Digite o caminho completo da pasta (ex: /home/usuario/Documentos):",
        ),
        InputTarget::SnapshotPath => (
            " Abrir Arquivo de Snapshot Externo (.tar.zst) ",
            "Digite o caminho completo do arquivo .tar.zst (ex: /tmp/backup.tar.zst):",
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Rgb(22, 27, 34)).fg(Color::Cyan));

    let content = vec![
        Line::from(vec![Span::raw(prompt_desc)]),
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
