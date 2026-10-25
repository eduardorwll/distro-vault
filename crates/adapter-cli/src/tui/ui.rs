use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

use super::app::{InputTarget, RestoreFocus, TuiApp, TuiMode, TuiRow};
use super::config_modal::ConfigField;
use super::explorer::{ExplorerEntryKind, ExplorerTarget, FileExplorerState};
use super::restore_tree::VisibleTreeNode;

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
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(Color::DarkGray),
        ),
        TuiMode::Restore => (
            Style::default().fg(Color::DarkGray),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    };

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(13, 17, 23)));
    let header_text = Paragraph::new(Line::from(vec![
        Span::styled(
            "distro-vault ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
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
                " [Tab] Alternar Aba  [↑/↓] Navegar  [Espaço] Alternar  [A] Explorador de Pastas  [Enter] Snapshot  [F3] Modo Recuperar  [Q] Sair ",
                Style::default().fg(Color::DarkGray),
            ),
            TuiMode::Restore => match app.restore_focus {
                RestoreFocus::SnapshotsList => Span::styled(
                    " [↑/↓] Snapshots  [Tab] Focar Árvore  [Enter/R] RESTAURAR  [S] SUB-DUMP  [O] Opções  [C] Pastas  [V] Densidade  [Q] Sair ",
                    Style::default().fg(Color::DarkGray),
                ),
                RestoreFocus::ItemTree => Span::styled(
                    " [↑/↓] Navegar Árvore  [Enter/→] Drop  [Espaço] Marcar  [Enter/R] RESTAURAR  [S] SUB-DUMP  [O] Opções  [C] Pastas  [V] Densidade  [Tab] Voltar ",
                    Style::default().fg(Color::Cyan),
                ),
            },
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

    // 6. Unified Interactive File & Directory Explorer Modal
    if app.file_explorer.is_open {
        render_file_explorer(f, &mut app.file_explorer);
    }

    // 7. Execution Options / Parameters Modal (Anti-Bloat)
    if app.options_modal.is_open {
        render_options_modal(f, app);
    }

    // 8. I/O Directories & Settings Modal
    if app.config_modal.is_open {
        render_config_modal(f, app);
    }
}

fn draw_backup_mode(f: &mut Frame, app: &mut TuiApp, tab_area: Rect, main_area: Rect) {
    let tab_titles = vec![
        format!(
            "[1] Apps & Usuário ({}/{})",
            app.count_selected_items(0),
            app.count_items(0)
        ),
        format!(
            "[2] Sistema Low-Level ({}/{})",
            app.count_selected_items(1),
            app.count_items(1)
        ),
        format!(
            "[3] Dependências ({}/{})",
            app.count_selected_items(2),
            app.count_items(2)
        ),
        format!(
            "[4] Skills ({}/{})",
            app.count_selected_items(3),
            app.count_items(3)
        ),
        format!(
            "[5] AppImages ({}/{})",
            app.count_selected_items(4),
            app.count_items(4)
        ),
        format!("[6] Pastas Extras ({})", app.custom_dirs.len()),
        format!(
            "[7] Zen Spaces ({}/{})",
            app.count_selected_items(6),
            app.count_items(6)
        ),
        format!(
            "[8] Chaves SSH ({}/{})",
            app.count_selected_items(7),
            app.count_items(7)
        ),
    ];

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Menus de Seleção do Snapshot "),
        )
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
                    let details = format!(
                        "{} arquivos • {:.2} MB",
                        dir.file_count,
                        dir.size_bytes as f64 / 1_048_576.0
                    );
                    render_item(&label, &details, true, is_selected)
                })
                .collect();
            (
                items,
                " 📁 Pastas Extras do Usuário (Pressione 'A' para adicionar) ",
            )
        }
        6 => (
            render_rows(&app.zen_spaces, current_cursor),
            " 🌐 Zen Browser Spaces & Workspaces ",
        ),
        7 => (
            render_rows(&app.ssh_keys, current_cursor),
            " 🔑 Chaves SSH Cadastradas (~/.ssh - Protegidas com Permissão Estrita 0700/0600) ",
        ),
        _ => (Vec::new(), ""),
    };

    let list_widget =
        List::new(list_items).block(Block::default().borders(Borders::ALL).title(title));

    let state = &mut app.list_states[app.current_tab];
    state.select(Some(current_cursor));
    f.render_stateful_widget(list_widget, main_area, state);
}

fn draw_restore_mode(f: &mut Frame, app: &mut TuiApp, sub_area: Rect, main_area: Rect) {
    let (pkg_label, pkg_color) = if app.restore_install_packages {
        ("SIM (Exige sudo no terminal)", Color::Yellow)
    } else {
        ("NÃO (Sem sudo)", Color::Green)
    };
    let dry_label = if app.restore_dry_run {
        "ATIVADO (Simulação)"
    } else {
        "DESATIVADO"
    };

    let staging_detected = app.detect_staging_for_selected();
    let staging_tag = if staging_detected.is_some() {
        "  •  ⚡ Cache Staging Ativo"
    } else {
        ""
    };

    let banner = Paragraph::new(Line::from(vec![
        Span::styled(
            " Diretório Padrão: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}  •  ", app.default_backups_dir.display()),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            "Snapshots Encontrados: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}  •  ", app.snapshots.len()),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            "[P] Pacotes: ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}  •  ", pkg_label),
            Style::default().fg(pkg_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "[D] Dry-Run: ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(dry_label, Style::default().fg(Color::White)),
        Span::styled(
            staging_tag,
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Painel de Controle de Recuperação "),
    );
    f.render_widget(banner, sub_area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(main_area);

    // Left Column: List of Snapshots
    let list_items: Vec<ListItem> = if app.snapshots.is_empty() {
        vec![
            ListItem::new(Line::from(vec![Span::styled(
                format!(
                    "  Nenhum snapshot encontrado em {}",
                    app.default_backups_dir.display()
                ),
                Style::default().fg(Color::Yellow),
            )])),
            ListItem::new(Line::from(vec![Span::styled(
                "  Pressione 'A' para abrir um arquivo .tar.zst manualmente.",
                Style::default().fg(Color::DarkGray),
            )])),
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
                    Span::styled(
                        snap.filename.clone(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  ({:.2} MB)", size_mb),
                        Style::default().fg(Color::Gray),
                    ),
                ]);
                let bg = if is_selected {
                    Color::Rgb(28, 42, 34)
                } else {
                    Color::Reset
                };
                ListItem::new(line).style(Style::default().bg(bg))
            })
            .collect()
    };

    let (left_border_color, left_title) = match app.restore_focus {
        RestoreFocus::SnapshotsList => (
            Color::Green,
            " 📂 Snapshots Disponíveis [Foco Ativo: ↑/↓ para navegar, Tab para Árvore] ",
        ),
        RestoreFocus::ItemTree => (
            Color::DarkGray,
            " 📂 Snapshots Disponíveis [Pressione Tab para Focar] ",
        ),
    };

    let list_widget = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(left_border_color))
            .title(left_title),
    );

    if !app.snapshots.is_empty() {
        app.snapshot_list_state.select(Some(app.snapshot_cursor));
    }
    f.render_stateful_widget(list_widget, cols[0], &mut app.snapshot_list_state);

    // Right Column: Interactive Drop Tree & Execution Parameters
    if let Some(snap) = app.selected_snapshot() {
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Metadados concisos do Snapshot
                Constraint::Min(8),    // Árvore de Seleção Granular ocupando toda a altura
            ])
            .split(cols[1]);

        let (distro_line, size_mb) = if let Some(ref m) = snap.manifest {
            (
                format!("{} (Host: {})", m.source_distro, m.source_host),
                snap.size_bytes as f64 / 1_048_576.0,
            )
        } else {
            (
                "Desconhecida".to_string(),
                snap.size_bytes as f64 / 1_048_576.0,
            )
        };

        let meta_lines = vec![
            Line::from(vec![
                Span::styled(
                    " Snapshot: ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &snap.filename,
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("  ({:.2} MB)", size_mb), Style::default().fg(Color::Gray)),
                Span::styled("  •  Origem: ", Style::default().fg(Color::Yellow)),
                Span::styled(distro_line, Style::default().fg(Color::White)),
                Span::styled("   [O] Opções  [C] Pastas", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![if staging_detected.is_some() {
                Span::styled(
                    " ⚡ Cache Staging Detectado: Restauração instantânea disponível sem descompactar!",
                    Style::default().fg(Color::LightGreen),
                )
            } else {
                Span::styled(
                    " 📦 Zstandard (.tar.zst)  •  [Enter/R] RESTAURAR  •  💾 [S] SUB-DUMP",
                    Style::default().fg(Color::DarkGray),
                )
            }]),
        ];
        let meta_block = Block::default()
            .borders(Borders::ALL)
            .title(" 🔍 Informações do Snapshot ");
        f.render_widget(Paragraph::new(meta_lines).block(meta_block), right_chunks[0]);

        // Chunk 1: Interactive Drop Tree
        let (tree_border_color, tree_title) = match app.restore_focus {
            RestoreFocus::ItemTree => (
                Color::Cyan,
                format!(
                    " 🗂️ Seleção Granular do Dump [Foco Ativo: [Espaço] Marcar • [Enter/→] Drop • [V] {} • [Tab] Voltar] ",
                    if app.vault_config.compact_view { "Modo Compacto" } else { "Modo Detalhado" }
                ),
            ),
            RestoreFocus::SnapshotsList => (
                Color::DarkGray,
                format!(
                    " 🗂️ Seleção Granular do Dump [Tab: Focar • O: Opções • C: Pastas • V: {}] ",
                    if app.vault_config.compact_view { "Compacto" } else { "Detalhado" }
                ),
            ),
        };

        let visible_nodes = app.restore_tree.visible_nodes();
        let tree_items: Vec<ListItem> = if visible_nodes.is_empty() {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "   Nenhum componente catalogado no manifesto deste snapshot.",
                Style::default().fg(Color::DarkGray),
            )]))]
        } else {
            visible_nodes
                .iter()
                .enumerate()
                .map(|(i, node)| {
                    let is_cursor =
                        app.restore_focus == RestoreFocus::ItemTree && i == app.restore_tree.cursor;
                    match node {
                        VisibleTreeNode::Category(cat_idx) => {
                            let cat = &app.restore_tree.categories[*cat_idx];
                            let drop_icon = if cat.is_expanded { "▼ " } else { "► " };
                            let check_box = if cat.is_selected {
                                Span::styled(
                                    "[x] ",
                                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                                )
                            } else {
                                Span::styled("[ ] ", Style::default().fg(Color::DarkGray))
                            };
                            let count_info = format!(
                                " ({}/{} selecionados)",
                                cat.selected_count(),
                                cat.items.len()
                            );
                            let line = Line::from(vec![
                                Span::styled(
                                    drop_icon,
                                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                                ),
                                check_box,
                                Span::styled(cat.icon, Style::default()),
                                Span::raw(" "),
                                Span::styled(
                                    &cat.title,
                                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(count_info, Style::default().fg(Color::Gray)),
                            ]);
                            let bg = if is_cursor {
                                Color::Rgb(35, 45, 60)
                            } else {
                                Color::Reset
                            };
                            ListItem::new(line).style(Style::default().bg(bg))
                        }
                        VisibleTreeNode::Item(cat_idx, item_idx) => {
                            let cat = &app.restore_tree.categories[*cat_idx];
                            let item = &cat.items[*item_idx];
                            let branch = if *item_idx + 1 == cat.items.len() {
                                "   └── "
                            } else {
                                "   ├── "
                            };
                            let check_box = if item.is_selected {
                                Span::styled(
                                    "[x] ",
                                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                                )
                            } else {
                                Span::styled("[ ] ", Style::default().fg(Color::DarkGray))
                            };
                            let sub = item.sublabel.as_deref().unwrap_or("");
                            let spans = if app.vault_config.compact_view || sub.is_empty() {
                                vec![
                                    Span::styled(branch, Style::default().fg(Color::DarkGray)),
                                    check_box,
                                    Span::styled(&item.label, Style::default().fg(Color::White)),
                                ]
                            } else {
                                vec![
                                    Span::styled(branch, Style::default().fg(Color::DarkGray)),
                                    check_box,
                                    Span::styled(&item.label, Style::default().fg(Color::White)),
                                    Span::styled(format!("  ({})", sub), Style::default().fg(Color::Gray)),
                                ]
                            };
                            let line = Line::from(spans);
                            let bg = if is_cursor {
                                Color::Rgb(35, 45, 60)
                            } else {
                                Color::Reset
                            };
                            ListItem::new(line).style(Style::default().bg(bg))
                        }
                    }
                })
                .collect()
        };

        let tree_widget = List::new(tree_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(tree_border_color))
                .title(tree_title),
        );

        if app.restore_focus == RestoreFocus::ItemTree && !visible_nodes.is_empty() {
            app.restore_tree
                .list_state
                .select(Some(app.restore_tree.cursor));
        } else {
            app.restore_tree.list_state.select(None);
        }
        f.render_stateful_widget(
            tree_widget,
            right_chunks[1],
            &mut app.restore_tree.list_state,
        );
    } else {
        let empty_p = Paragraph::new(vec![
            Line::from(""),
            Line::from("  Nenhum snapshot selecionado."),
            Line::from("  Use as setas [↑/↓] para navegar ou pressione 'A' para adicionar um arquivo .tar.zst."),
        ])
        .block(Block::default().borders(Borders::ALL).title(" 🔍 Inspeção "));
        f.render_widget(empty_p, cols[1]);
    }
}

fn render_rows(rows: &[TuiRow], current_cursor: usize) -> Vec<ListItem<'static>> {
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected = i == current_cursor;
            match row {
                TuiRow::Header {
                    category_id,
                    title,
                    count,
                } => {
                    let selected_count = rows
                        .iter()
                        .filter(|r| match r {
                            TuiRow::Item {
                                category_id: c,
                                item,
                            } => c == category_id && item.included,
                            _ => false,
                        })
                        .count();
                    render_header(title, selected_count, *count, is_selected)
                }
                TuiRow::Item { item, .. } => {
                    render_item(&item.name, &item.details, item.included, is_selected)
                }
            }
        })
        .collect()
}

fn render_header(
    title: &str,
    selected: usize,
    total: usize,
    is_selected: bool,
) -> ListItem<'static> {
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
        Span::styled(
            prefix.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            header_label,
            Style::default().fg(text_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(fill, Style::default().fg(Color::DarkGray)),
    ]);

    ListItem::new(line).style(Style::default().bg(bg_color))
}

fn render_item(name: &str, details: &str, included: bool, is_selected: bool) -> ListItem<'static> {
    let (mark, mark_style) = if included {
        (
            "[x] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
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
        Span::styled(
            name.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
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
            Span::styled(
                buffer,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "_",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
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

fn render_file_explorer(f: &mut Frame, explorer: &mut FileExplorerState) {
    let area = centered_rect(80, 75, f.area());
    f.render_widget(Clear, area);

    let (title, help_action) = match explorer.target {
        ExplorerTarget::CustomDir => (
            " 📂 Navegador: Selecionar Pasta para Backup ",
            "[Espaço/S] Selecionar Esta Pasta para Backup",
        ),
        ExplorerTarget::SnapshotOrFolder => (
            " 📂 Navegador: Selecionar Arquivo (.tar.zst) ou Pasta de Backups ",
            "[Espaço/S] Escanear Todos os Backups Desta Pasta",
        ),
        ExplorerTarget::OutputDir => (
            " 📁 Navegador: Selecionar Diretório Alvo para Backups/Slices ",
            "[Espaço/S] Definir Como Diretório Padrão de Saída",
        ),
        ExplorerTarget::InputDir => (
            " 📂 Navegador: Selecionar Diretório de Origem de Snapshots ",
            "[Espaço/S] Adicionar Diretório à Lista de Varredura",
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(18, 22, 28)).fg(Color::White));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Path header
            Constraint::Min(5),    // List of files/directories
            Constraint::Length(2), // Instructions / Footer
        ])
        .split(block.inner(area));

    f.render_widget(block, area);

    // Current path header
    let path_line = Line::from(vec![
        Span::styled(
            " 📍 Diretório: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            explorer.current_dir.display().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(path_line), inner_chunks[0]);

    // Items list
    let items: Vec<ListItem> = if explorer.entries.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "   (Diretório vazio ou sem arquivos compatíveis)",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        explorer
            .entries
            .iter()
            .map(|entry| {
                let line = match entry.kind {
                    ExplorerEntryKind::ParentDir => Line::from(vec![
                        Span::styled(" 📁 ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            &entry.name,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    ExplorerEntryKind::Directory => Line::from(vec![
                        Span::styled(" 📁 ", Style::default().fg(Color::Cyan)),
                        Span::styled(&entry.name, Style::default().fg(Color::White)),
                        Span::styled("/", Style::default().fg(Color::DarkGray)),
                    ]),
                    ExplorerEntryKind::SnapshotArchive => {
                        let size_mb = (entry.size_bytes as f64) / 1024.0 / 1024.0;
                        Line::from(vec![
                            Span::styled(" 📦 ", Style::default().fg(Color::Green)),
                            Span::styled(
                                &entry.name,
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!(" ({:.1} MB)", size_mb),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ])
                    }
                };
                ListItem::new(line)
            })
            .collect()
    };

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(35, 45, 60))
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, inner_chunks[1], &mut explorer.list_state);

    // Explorer footer help
    let help_line = Line::from(vec![Span::styled(
        format!(
            " [↑/↓/j/k] Navegar  •  [Enter] Abrir  •  {}  •  [Backspace] Subir  •  [Esc/Q] Fechar ",
            help_action
        ),
        Style::default().fg(Color::DarkGray),
    )]);
    f.render_widget(
        Paragraph::new(help_line).alignment(Alignment::Center),
        inner_chunks[2],
    );
}

fn render_options_modal(f: &mut Frame, app: &TuiApp) {
    let area = centered_rect(60, 35, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " ⚙️ Opções & Parâmetros de Execução ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(18, 22, 28)).fg(Color::White));

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Intro
            Constraint::Length(6), // Toggles list
            Constraint::Length(2), // Footer
        ])
        .margin(1)
        .split(area);

    f.render_widget(block, area);

    let intro = Paragraph::new(Line::from(vec![
        Span::styled(
            "Defina os parâmetros operacionais antes de iniciar a restauração ou sub-dump:",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(intro, inner[0]);

    let (pkg_check, pkg_color) = if app.restore_install_packages {
        ("[X] ATIVADO", Color::Green)
    } else {
        ("[ ] DESATIVADO", Color::DarkGray)
    };

    let (dry_check, dry_color) = if app.restore_dry_run {
        ("[X] MODO SIMULAÇÃO (DRY-RUN)", Color::Yellow)
    } else {
        ("[ ] RESTAURAÇÃO REAL NO SISTEMA", Color::White)
    };

    let (view_check, view_color) = if app.vault_config.compact_view {
        ("[X] MODO COMPACTO (Alta Densidade)", Color::Cyan)
    } else {
        ("[ ] MODO EXPANDIDO (Detalhes & Sublabels)", Color::White)
    };

    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  [P] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Instalar Pacotes Nativos: ", Style::default().fg(Color::White)),
            Span::styled(pkg_check, Style::default().fg(pkg_color).add_modifier(Modifier::BOLD)),
            Span::styled(" (pacman / flatpak / apt / dnf)", Style::default().fg(Color::DarkGray)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  [D] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Simulação Dry-Run:       ", Style::default().fg(Color::White)),
            Span::styled(dry_check, Style::default().fg(dry_color).add_modifier(Modifier::BOLD)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  [V] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Densidade da Árvore:    ", Style::default().fg(Color::White)),
            Span::styled(view_check, Style::default().fg(view_color).add_modifier(Modifier::BOLD)),
        ])),
    ];

    let list = List::new(items);
    f.render_widget(list, inner[1]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [P/D/V] Alternar Opções  •  [Esc/Enter/Q] Fechar Janela ",
            Style::default().fg(Color::DarkGray),
        ),
    ])).alignment(Alignment::Center);
    f.render_widget(help, inner[2]);
}

fn render_config_modal(f: &mut Frame, app: &TuiApp) {
    let area = centered_rect(75, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " 📁 Gerenciador de Diretórios & Configuração (~/.config/distro-vault/config.toml) ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(18, 22, 28)).fg(Color::White));

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Output Dir Box
            Constraint::Min(8),    // Input Dirs List
            Constraint::Length(3), // Compact View Box
            Constraint::Length(3), // Save / Action Buttons
            Constraint::Length(2), // Help Footer
        ])
        .margin(1)
        .split(area);

    f.render_widget(block, area);

    let modal = &app.config_modal;

    // Field 1: Output Directory
    let output_border_style = if modal.focused_field == ConfigField::OutputDir {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let output_block = Block::default()
        .borders(Borders::ALL)
        .title(" [1] Diretório Padrão de Saída (Dumps & Slices) [Tecla O p/ Explorador] ")
        .border_style(output_border_style);
    let output_text = Paragraph::new(Line::from(vec![
        Span::styled("  📁 ", Style::default().fg(Color::Cyan)),
        Span::styled(
            modal.pending_output_dir.display().to_string(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ])).block(output_block);
    f.render_widget(output_text, inner[0]);

    // Field 2: Input Dirs List
    let input_border_style = if modal.focused_field == ConfigField::InputDirsList {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " [2] Diretórios de Entrada para Varredura de Snapshots ({}) [+/A: Adicionar | D/Del: Remover] ",
            modal.pending_input_dirs.len()
        ))
        .border_style(input_border_style);

    let input_items: Vec<ListItem> = if modal.pending_input_dirs.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("    Nenhum diretório configurado. Pressione [+] ou [A] para adicionar.", Style::default().fg(Color::DarkGray)),
        ]))]
    } else {
        modal
            .pending_input_dirs
            .iter()
            .enumerate()
            .map(|(idx, dir)| {
                let is_selected = modal.focused_field == ConfigField::InputDirsList && idx == modal.input_dir_cursor;
                let (prefix, style) = if is_selected {
                    (" ▶ 📂 ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                } else {
                    ("   📂 ", Style::default().fg(Color::White))
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(dir.display().to_string(), style),
                ]))
            })
            .collect()
    };
    let input_list = List::new(input_items).block(input_block);
    f.render_widget(input_list, inner[1]);

    // Field 3: Compact View Toggle
    let compact_border_style = if modal.focused_field == ConfigField::CompactViewToggle {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let compact_block = Block::default()
        .borders(Borders::ALL)
        .title(" [3] Densidade Padrão da Interface [Espaço para Alternar] ")
        .border_style(compact_border_style);
    let (compact_status, compact_color) = if modal.pending_compact_view {
        ("[X] MODO COMPACTO (Alta Densidade de Itens)", Color::Cyan)
    } else {
        ("[ ] MODO EXPANDIDO (Visualização Clássica com Subtítulos)", Color::White)
    };
    let compact_text = Paragraph::new(Line::from(vec![
        Span::styled("   ", Style::default()),
        Span::styled(compact_status, Style::default().fg(compact_color).add_modifier(Modifier::BOLD)),
    ])).block(compact_block);
    f.render_widget(compact_text, inner[2]);

    // Field 4: Save & Apply Button
    let save_border_style = if modal.focused_field == ConfigField::SaveAndClose {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let save_block = Block::default()
        .borders(Borders::ALL)
        .border_style(save_border_style);
    let save_text = Paragraph::new(Line::from(vec![
        Span::styled(
            if modal.focused_field == ConfigField::SaveAndClose {
                " ▶ [S / Enter] SALVAR E APLICAR PREFERÊNCIAS NO DISCO ◀ "
            } else {
                "   [S / Enter] SALVAR E APLICAR PREFERÊNCIAS NO DISCO   "
            },
            if modal.focused_field == ConfigField::SaveAndClose {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ),
    ]))
    .alignment(Alignment::Center)
    .block(save_block);
    f.render_widget(save_text, inner[3]);

    // Field 5: Footer Help
    let help_line = Line::from(vec![
        Span::styled(
            " [Tab/↑/↓] Campo  •  [O] Mudar Saída  •  [+/A] Add Entrada  •  [D/Del] Remover  •  [S] Salvar  •  [Esc] Cancelar ",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    f.render_widget(
        Paragraph::new(help_line).alignment(Alignment::Center),
        inner[4],
    );
}
