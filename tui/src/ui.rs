use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};

use crate::create_field::CreateField;
use crate::dialogs::{ConfirmDialog, PkgDialog};
use crate::tabs::Tab;
use crate::{app::App, dialogs::HelpDialog};

/// Draw the TUI to the given frame
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title + tabs
            Constraint::Min(0),    // content
            Constraint::Length(1), // status bar
        ])
        .split(frame.area());

    draw_tabs(frame, app, chunks[0]);

    match app.tab {
        Tab::Environments => draw_environments(frame, app, chunks[1]),
        Tab::UvInfo => draw_uv_info(frame, app, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);

    // Dialog overlay – rendered last so it appears on top of everything else.
    if let Some(ref dialog) = app.create_dialog {
        draw_create_dialog(frame, dialog);
    }
    if let Some(ref dialog) = app.confirm_dialog {
        draw_confirm_dialog(frame, dialog);
    }
    if let Some(ref dialog) = app.pkg_dialog {
        let venv_name = if !app.venvs.is_empty() {
            app.venvs[app.selected].name.as_ref()
        } else {
            ""
        };
        draw_pkg_dialog(frame, dialog, venv_name);
    }
    if let Some(ref dialog) = app.help_dialog {
        draw_help_dialog(frame, dialog);
    }
}
fn draw_tabs(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let tab_titles: Vec<Line> = Tab::ALL.iter().map(|t| Line::from(t.title())).collect();

    let selected = Tab::ALL.iter().position(|t| *t == app.tab).unwrap_or(0);

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Pylot TUI ")
                .title_alignment(Alignment::Center),
        )
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn draw_environments(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Split horizontally: venv list (left) + detail panel (right)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // ── Left: venv list with column header ───────────────────────────────────
    let title = format!(" Virtual Environments ({}) ", app.venvs.len());
    let outer_block = Block::default().borders(Borders::ALL).title(title);
    let inner_area = outer_block.inner(columns[0]);
    frame.render_widget(outer_block, columns[0]);

    // Split the inner area: 1-line column header + list
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner_area);

    let header_style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    let header = Paragraph::new(Line::from(vec![
        // 7 chars offset matches: 2 (highlight prefix) + 3 (index field) + 2 (". ")
        Span::styled("       ", header_style),
        Span::styled(format!("{:<22}", "Name"), header_style),
        Span::styled("  Version", header_style),
    ]));
    frame.render_widget(header, left_chunks[0]);

    let items: Vec<ListItem> = app
        .venvs
        .iter()
        .enumerate()
        .map(|(i, venv)| {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:>3}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<22}", venv.name.as_ref()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("  {}", venv.python_version),
                    Style::default().fg(Color::Green),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !app.venvs.is_empty() {
        state.select(Some(app.selected));
    }

    frame.render_stateful_widget(list, left_chunks[1], &mut state);

    // ── Right: detail panel ──────────────────────────────────────────────────
    draw_venv_detail(frame, app, columns[1]);
}

fn draw_venv_detail(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let label_style = Style::default().fg(Color::DarkGray);

    if app.venvs.is_empty() {
        let lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled("  No environments found.", label_style)]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Press [n] to create one.",
                label_style,
            )]),
        ];
        frame.render_widget(
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Details ")),
            area,
        );
        return;
    }

    let venv = &app.venvs[app.selected];

    // Render the outer block, using the venv name as the title.
    let block_title = format!(" {} ", venv.name);
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(block_title)
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Split inner area: fixed metadata section + packages list
    let meta_height = 6u16; // python + blank + location + blank + packages header + divider
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(meta_height), Constraint::Min(0)])
        .split(inner);

    let display_path = pylot_shared::utils::shorten_home_path(venv.path.as_str());
    let version_text = if venv.python_version.is_empty() {
        "unknown".to_string()
    } else {
        venv.python_version.clone()
    };

    // Build the packages header and list, accounting for search mode.
    let search_active = app.pkg_search.is_some();
    let search_query = app.pkg_search.as_deref().unwrap_or("");
    let query_lower = search_query.to_lowercase();

    let filtered_packages: Vec<&String> = if search_active && !search_query.is_empty() {
        venv.installed_packages
            .iter()
            .filter(|p| p.to_lowercase().contains(&query_lower))
            .collect()
    } else {
        venv.installed_packages.iter().collect()
    };

    let total_pkg_count = venv.installed_packages.len();
    let display_count = if search_active && !search_query.is_empty() {
        format!(
            "  Packages ({}/{})  ",
            filtered_packages.len(),
            total_pkg_count
        )
    } else {
        format!("  Packages ({})  ", total_pkg_count)
    };

    let pkg_scroll_hint = if search_active {
        format!("/{}█", search_query)
    } else {
        "[j] down  [k] up".to_string()
    };

    let divider = "─".repeat(inner_chunks[0].width.saturating_sub(2) as usize);

    let meta_lines = vec![
        Line::from(vec![
            Span::styled("  Python   : ", label_style),
            Span::styled(version_text, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Location : ", label_style),
            Span::styled(display_path, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                display_count,
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            if search_active {
                Span::styled(
                    pkg_scroll_hint,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(pkg_scroll_hint, label_style)
            },
        ]),
        Line::from(Span::styled(divider, label_style)),
    ];
    frame.render_widget(Paragraph::new(meta_lines), inner_chunks[0]);

    // Packages list (scrollable via pkg_scroll, filtered/highlighted when search active)
    let pkg_items: Vec<ListItem> = filtered_packages
        .iter()
        .map(|p| {
            // "name version" → render with optional search highlight
            let line = match p.splitn(2, ' ').collect::<Vec<_>>().as_slice() {
                [name, version] => {
                    if search_active && !search_query.is_empty() {
                        // Highlight matches in name and version
                        let name_spans = highlight_match(
                            &format!("  {}", name),
                            &query_lower,
                            Color::Magenta,
                            Color::Yellow,
                        );
                        let version_spans = highlight_match(
                            &format!(" {}", version),
                            &query_lower,
                            Color::DarkGray,
                            Color::Yellow,
                        );
                        let mut spans = name_spans;
                        spans.extend(version_spans);
                        Line::from(spans)
                    } else {
                        Line::from(vec![
                            Span::styled(
                                format!("  {}", name),
                                Style::default().fg(Color::Magenta),
                            ),
                            Span::styled(
                                format!(" {}", version),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ])
                    }
                }
                _ => Line::from(Span::styled(
                    format!("  {}", p),
                    Style::default().fg(Color::Magenta),
                )),
            };
            ListItem::new(line)
        })
        .collect();

    let pkg_list = List::new(pkg_items);
    let pkg_scroll = if search_active { 0 } else { app.pkg_scroll };
    let mut pkg_state = ListState::default().with_offset(pkg_scroll);
    frame.render_stateful_widget(pkg_list, inner_chunks[1], &mut pkg_state);
}

/// Split `text` into styled spans, highlighting occurrences of `query` (already lowercase).
/// Matched portions use `highlight_color`; the rest uses `base_color`.
fn highlight_match<'a>(
    text: &str,
    query: &str,
    base_color: Color,
    highlight_color: Color,
) -> Vec<Span<'a>> {
    if query.is_empty() {
        return vec![Span::styled(
            text.to_string(),
            Style::default().fg(base_color),
        )];
    }
    let mut spans = Vec::new();
    let text_lower = text.to_lowercase();
    let mut last = 0;
    while let Some(pos) = text_lower[last..].find(query) {
        let abs = last + pos;
        if abs > last {
            spans.push(Span::styled(
                text[last..abs].to_string(),
                Style::default().fg(base_color),
            ));
        }
        spans.push(Span::styled(
            text[abs..abs + query.len()].to_string(),
            Style::default()
                .fg(highlight_color)
                .add_modifier(Modifier::BOLD),
        ));
        last = abs + query.len();
    }
    if last < text.len() {
        spans.push(Span::styled(
            text[last..].to_string(),
            Style::default().fg(base_color),
        ));
    }
    spans
}

fn draw_uv_info(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let status_style = if app.uv_installed {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };

    let status_text = if app.uv_installed {
        "Installed"
    } else {
        "Not installed"
    };

    // Extract bare semver from "uv X.Y.Z (hash date)" for comparison.
    let current_semver: Option<&str> = app.uv_version.as_deref().and_then(|v| {
        // "uv 0.5.0 (abc 2024-01-01)" → "0.5.0"
        v.split_whitespace().nth(1)
    });

    let update_available = matches!(
        (current_semver, app.uv_latest_version.as_deref()),
        (Some(cur), Some(latest)) if cur != latest
    );

    let loading = app.is_uv_info_loading();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Status:   "),
            Span::styled(status_text, status_style.add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Version:  "),
            Span::styled(
                if loading {
                    "..."
                } else {
                    app.uv_version.as_deref().unwrap_or("N/A")
                },
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Latest:   "),
            Span::styled(
                if loading {
                    "..."
                } else {
                    app.uv_latest_version.as_deref().unwrap_or("N/A")
                },
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
    ];

    if update_available {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "⬆ Update available",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    " ({})",
                    app.uv_latest_version.as_deref().unwrap_or("")
                ),
                Style::default().fg(Color::Yellow),
            ),
        ]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(vec![Span::styled(
        "  Actions:  ",
        Style::default().fg(Color::DarkGray),
    )]));

    if app.uv_installed {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                "[u]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Update    "),
            Span::styled(
                "[d]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Uninstall"),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                "[i]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Install"),
        ]));
    }

    let paragraph =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Astral UV "));

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Priority 1: show a one-shot status message (success or error from a background task).
    if let Some((ref msg, is_error, set_at)) = app.status_message {
        let color = if is_error { Color::Red } else { Color::Green };
        let remaining = 3u64.saturating_sub(set_at.elapsed().as_secs());
        let hint = format!("  (auto-dismisses in {}s)", remaining);
        let spans = vec![
            Span::styled(
                if is_error { " ✗ " } else { " ✓ " },
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(msg.as_str(), Style::default().fg(color)),
            Span::styled(hint, Style::default().fg(Color::DarkGray)),
        ];
        let bar = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(bar, area);
        return;
    }

    // Priority 2: show the background task name while a task is running.
    if let Some(ref task_name) = app.bg_task_name {
        let spans = vec![
            Span::styled(
                " ⏳ Running: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(task_name.as_str(), Style::default().fg(Color::Cyan)),
            Span::styled("…", Style::default().fg(Color::Yellow)),
            Span::styled("  (q: quit)", Style::default().fg(Color::DarkGray)),
        ];
        let bar = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(bar, area);
        return;
    }

    // Priority 3: show confirm-dialog hints when a confirmation is pending.
    if app.confirm_dialog.is_some() {
        let spans = vec![
            Span::styled(
                "y / Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": confirm  "),
            Span::styled(
                "n / Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(": cancel"),
        ];
        let help = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(help, area);
        return;
    }

    // When the create dialog is open, show dialog-specific hints instead of the normal bar.
    if let Some(ref d) = app.create_dialog {
        let completions_active =
            d.field == crate::create_field::CreateField::ReqFile && !d.completions.is_empty();
        let spans = if completions_active {
            vec![
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(": navigate completions  "),
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(": complete  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(": dismiss"),
            ]
        } else {
            vec![
                Span::styled("Tab", Style::default().fg(Color::Yellow)),
                Span::raw(": change field  "),
                Span::styled("←→", Style::default().fg(Color::Yellow)),
                Span::raw(": cursor  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(": confirm  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(": cancel"),
            ]
        };
        let help = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(help, area);
        return;
    }

    // When the pkg_dialog is open, show dialog-specific hints.
    if app.pkg_dialog.is_some() {
        let action = app.pkg_dialog.as_ref().map(|d| d.title()).unwrap_or("");
        let spans = vec![
            Span::styled(
                action,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": confirm  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": cancel"),
        ];
        let help = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(help, area);
        return;
    }

    // When package search is active, show search hints (including other venvs).
    if let Some(ref query) = app.pkg_search {
        let query_lower = query.to_lowercase();
        let other_venvs: Vec<&str> = if !query.is_empty() {
            let current = app.selected;
            app.venvs
                .iter()
                .enumerate()
                .filter(|(i, v)| {
                    *i != current
                        && v.installed_packages
                            .iter()
                            .any(|p| p.to_lowercase().contains(&query_lower))
                })
                .map(|(_, v)| v.name.as_ref())
                .collect()
        } else {
            vec![]
        };

        let mut spans = vec![
            Span::styled(
                "/",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": search  "),
            Span::styled("Enter/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": close"),
        ];
        if !other_venvs.is_empty() {
            spans.push(Span::styled(
                "  also in: ",
                Style::default().fg(Color::DarkGray),
            ));
            spans.push(Span::styled(
                other_venvs.join(", "),
                Style::default().fg(Color::Cyan),
            ));
        }
        let help = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(help, area);
        return;
    }

    let mut spans = vec![
        Span::styled(" Tab", Style::default().fg(Color::Yellow)),
        Span::raw(": switch tab  "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(": navigate  "),
        Span::styled("?", Style::default().fg(Color::Yellow)),
        Span::raw(": help  "),
    ];

    match app.tab {
        Tab::Environments => {
            spans.push(Span::styled("n", Style::default().fg(Color::Yellow)));
            spans.push(Span::raw(": new  "));
            if !app.venvs.is_empty() {
                spans.push(Span::styled("d", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(": delete  "));
                spans.push(Span::styled("Enter", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(": activate  "));
                spans.push(Span::styled("a / r", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(": add / remove pkgs  "));
            }
        }
        Tab::UvInfo => {
            if app.uv_installed {
                spans.push(Span::styled("u", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(": update  "));
                spans.push(Span::styled("d", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(": uninstall  "));
            } else {
                spans.push(Span::styled("i", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(": install  "));
            }
        }
    }

    let help = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(help, area);
}

/// Maximum completions visible at once in the completion dropdown.
const COMPLETION_MAX_SHOWN: usize = 6;

/// Render the create-venv dialog as a centered overlay popup.
fn draw_create_dialog(frame: &mut Frame, dialog: &crate::create_dialog::CreateDialog) {
    let completions_active = dialog.field == CreateField::ReqFile && !dialog.completions.is_empty();

    // Compute the visible window into the completions list.
    let total = dialog.completions.len();
    let scroll = dialog.completion_scroll;
    let more_above = scroll > 0;
    let visible_end = (scroll + COMPLETION_MAX_SHOWN).min(total);
    let shown_count = if completions_active {
        visible_end.saturating_sub(scroll)
    } else {
        0
    };
    let more_below = completions_active && visible_end < total;

    // Extra lines: 1 blank separator + shown rows + optional scroll indicators.
    let extra_height = if completions_active {
        let rows = shown_count
            + 1 // blank separator
            + if more_above { 1 } else { 0 }
            + if more_below { 1 } else { 0 };
        rows as u16
    } else {
        0
    };
    let area = centered_rect(60, 17 + extra_height, frame.area());

    // Clear the background so the dialog appears cleanly over other widgets.
    frame.render_widget(Clear, area);

    let focused_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);
    let hint_style = Style::default().fg(Color::DarkGray);

    // Helper: returns the style for a label based on whether the field is focused.
    let label_style = |field: CreateField| {
        if dialog.field == field {
            focused_style
        } else {
            normal_style
        }
    };

    let default_indicator = if dialog.default_pkgs { "[x]" } else { "[ ]" };

    // Build req_file line with cursor at the correct position.
    let req_before: String = dialog
        .req_file
        .chars()
        .take(dialog.req_file_cursor)
        .collect();
    let req_after: String = dialog
        .req_file
        .chars()
        .skip(dialog.req_file_cursor)
        .collect();

    let mut req_file_spans = vec![
        Span::styled("  Req. file   : ", label_style(CreateField::ReqFile)),
        Span::styled(req_before, Style::default().fg(Color::Blue)),
    ];
    if dialog.field == CreateField::ReqFile {
        req_file_spans.push(Span::styled("█", Style::default().fg(Color::Blue)));
    }
    req_file_spans.push(Span::styled(req_after, Style::default().fg(Color::Blue)));

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Name        : ", label_style(CreateField::Name)),
            Span::styled(dialog.name.as_str(), Style::default().fg(Color::Cyan)),
            if dialog.field == CreateField::Name {
                Span::styled("█", Style::default().fg(Color::Cyan))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Python ver. : ", label_style(CreateField::Version)),
            Span::styled(dialog.version.as_str(), Style::default().fg(Color::Green)),
            if dialog.field == CreateField::Version {
                Span::styled("█", Style::default().fg(Color::Green))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Packages    : ", label_style(CreateField::Packages)),
            Span::styled(
                dialog.packages.as_str(),
                Style::default().fg(Color::Magenta),
            ),
            if dialog.field == CreateField::Packages {
                Span::styled("█", Style::default().fg(Color::Magenta))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(vec![
            Span::raw("                "),
            Span::styled("e.g. requests,flask==2.28.0", hint_style),
        ]),
        Line::from(""),
        Line::from(req_file_spans),
        Line::from(vec![
            Span::raw("                "),
            Span::styled("path to requirements.txt (optional)", hint_style),
        ]),
    ];

    // Show directory completions with scroll indicators.
    if completions_active {
        lines.push(Line::from(""));

        // ▲ indicator when entries are scrolled past above.
        if more_above {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("▲ {} more above", scroll), hint_style),
            ]));
        }

        // Visible completion entries.
        for i in scroll..visible_end {
            let entry = &dialog.completions[i];
            let is_selected = i == dialog.completion_selected;
            let (prefix, entry_style) = if is_selected {
                (
                    "  ▶ ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("    ", Style::default().fg(Color::Blue))
            };
            lines.push(Line::from(vec![
                Span::styled(prefix, entry_style),
                Span::styled(entry.as_str(), entry_style),
            ]));
        }

        // ▼ indicator when more entries exist below the visible window.
        if more_below {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("▼ {} more below", total - visible_end), hint_style),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Default pkgs: ", label_style(CreateField::DefaultPkgs)),
        Span::styled(default_indicator, label_style(CreateField::DefaultPkgs)),
        Span::raw("  "),
        Span::styled("(Space to toggle)", hint_style),
    ]));
    lines.push(Line::from(""));
    let footer_line = if completions_active {
        Line::from(vec![
            Span::styled("  ↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(": navigate  "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": complete  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": dismiss"),
        ])
    } else {
        Line::from(vec![
            Span::styled("  Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": change field  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": confirm  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": cancel"),
        ])
    };
    lines.push(footer_line);

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" New Virtual Environment ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(paragraph, area);
}

/// Render the add/remove-package dialog as a centered overlay popup.
fn draw_pkg_dialog(frame: &mut Frame, dialog: &PkgDialog, venv_name: &str) {
    let area = centered_rect(60, 10, frame.area());

    frame.render_widget(Clear, area);

    let hint_style = Style::default().fg(Color::DarkGray);
    let input_color = match dialog.mode {
        crate::dialogs::PkgDialogMode::Add => Color::Green,
        crate::dialogs::PkgDialogMode::Remove => Color::Red,
    };
    let border_color = input_color;

    let venv_label = if venv_name.is_empty() {
        "".to_string()
    } else {
        format!("  Venv: {}", venv_name)
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            venv_label,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Packages    : ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(dialog.input.as_str(), Style::default().fg(input_color)),
            Span::styled("█", Style::default().fg(input_color)),
        ]),
        Line::from(vec![
            Span::raw("                "),
            Span::styled("e.g. requests,flask==2.28.0", hint_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": confirm  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(dialog.title())
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(paragraph, area);
}

/// Render the yes/no confirmation dialog as a centered overlay popup.
fn draw_confirm_dialog(frame: &mut Frame, dialog: &ConfirmDialog) {
    let area = centered_rect(52, 7, frame.area());

    // Clear the background so the dialog appears cleanly over other widgets.
    frame.render_widget(Clear, area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                dialog.message(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "This action cannot be undone.",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[y] Yes",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                "[n] No",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Confirm ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Red)),
    );

    frame.render_widget(paragraph, area);
}

/// Render the help dialog as a centered overlay popup.
fn draw_help_dialog(frame: &mut Frame, dialog: &HelpDialog) {
    let area = centered_rect(dialog.width, dialog.height, frame.area());

    // Clear the background so the dialog appears cleanly over other widgets.
    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(dialog.lines()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::LightGreen)),
    );

    frame.render_widget(paragraph, area);
}
/// Return a `Rect` centered within `r` with the given width (columns) and height (rows).
///
/// If the requested size exceeds the available space it is clamped to fit.
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let w = width.min(r.width);
    let h = height.min(r.height);
    let x = r.x + r.width.saturating_sub(w) / 2;
    let y = r.y + r.height.saturating_sub(h) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::ConfirmAction;
    use crate::app::App;
    use crate::create_dialog::CreateDialog;
    use crate::dialogs::ConfirmDialog;
    use pylot_shared::uvvenv::UvVenv;
    use ratatui::{backend::TestBackend, Terminal};
    use std::borrow::Cow;
    use std::time::Instant;

    fn make_app<'a>() -> App<'a> {
        App::new(vec![], true, Some("uv 0.5.0".to_string()))
    }

    fn make_app_with_venvs<'a>() -> App<'a> {
        let venvs = vec![
            UvVenv::new(
                Cow::Owned("env1".to_string()),
                "".to_string(),
                "3.11".to_string(),
                vec![],
                false,
            ),
            UvVenv::new(
                Cow::Owned("env2".to_string()),
                "".to_string(),
                "3.12".to_string(),
                vec![],
                false,
            ),
        ];
        App::new(venvs, true, Some("uv 0.5.0".to_string()))
    }

    // ── centered_rect ────────────────────────────────────────────────────────

    #[test]
    fn test_centered_rect_basic() {
        let r = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        let result = centered_rect(60, 14, r);
        assert_eq!(result.width, 60);
        assert_eq!(result.height, 14);
        // x should be centered: (100 - 60) / 2 = 20
        assert_eq!(result.x, 20);
        // y should be centered: (50 - 14) / 2 = 18
        assert_eq!(result.y, 18);
    }

    #[test]
    fn test_centered_rect_clamped_when_larger_than_parent() {
        let r = Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 10,
        };
        let result = centered_rect(100, 50, r);
        // Should be clamped to the parent's dimensions.
        assert_eq!(result.width, 40);
        assert_eq!(result.height, 10);
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
    }

    #[test]
    fn test_centered_rect_zero_size() {
        let r = Rect {
            x: 5,
            y: 3,
            width: 80,
            height: 24,
        };
        let result = centered_rect(0, 0, r);
        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    #[test]
    fn test_centered_rect_offset_parent() {
        let r = Rect {
            x: 10,
            y: 5,
            width: 80,
            height: 24,
        };
        let result = centered_rect(40, 10, r);
        assert_eq!(result.width, 40);
        assert_eq!(result.height, 10);
        // x: 10 + (80 - 40) / 2 = 10 + 20 = 30
        assert_eq!(result.x, 30);
        // y: 5 + (24 - 10) / 2 = 5 + 7 = 12
        assert_eq!(result.y, 12);
    }

    // ── draw – Environments tab (empty) ──────────────────────────────────────

    #[test]
    fn test_draw_environments_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = make_app();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    // ── draw – Environments tab with venvs ───────────────────────────────────

    #[test]
    fn test_draw_environments_with_venvs() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = make_app_with_venvs();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    // ── draw – UV Info tab ───────────────────────────────────────────────────

    #[test]
    fn test_draw_uv_info_installed() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app();
        app.next_tab(); // switch to UvInfo
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_uv_info_not_installed() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(vec![], false, None);
        app.next_tab();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_uv_info_update_available() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(vec![], true, Some("uv 0.5.0 (abc 2024-01-01)".to_string()));
        app.uv_latest_version = Some("0.6.0".to_string());
        app.next_tab();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_uv_info_up_to_date() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(vec![], true, Some("uv 0.6.0 (abc 2024-06-01)".to_string()));
        app.uv_latest_version = Some("0.6.0".to_string());
        app.next_tab();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_uv_info_loading() {
        // While the background UV info task is in-flight, version fields show "...".
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(vec![], true, None);
        // Simulate a pending uv_info_rx by creating a channel whose sender we
        // immediately drop – the receiver will be Closed on try_recv, but
        // is_uv_info_loading() returns true while the Option is Some.
        let (_tx, rx) = tokio::sync::oneshot::channel::<(Option<String>, Option<String>)>();
        app.uv_info_rx = Some(rx);
        assert!(app.is_uv_info_loading());
        app.next_tab();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    // ── draw – status messages ───────────────────────────────────────────────

    #[test]
    fn test_draw_with_status_message_ok() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app();
        app.status_message = Some(("Operation completed.".to_string(), false, Instant::now()));
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_with_status_message_error() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app();
        app.status_message = Some(("Something went wrong.".to_string(), true, Instant::now()));
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_with_bg_task_running() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app();
        app.bg_task_name = Some("Installing UV".to_string());
        // Simulate a busy state by setting a receiver.
        let (_tx, rx) = tokio::sync::oneshot::channel::<Result<(), String>>();
        app.bg_rx = Some(rx);
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    // ── draw – confirm dialog overlay ────────────────────────────────────────

    #[test]
    fn test_draw_with_confirm_dialog_delete_venv() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app();
        app.confirm_dialog = Some(ConfirmDialog::new(ConfirmAction::DeleteVenv(
            "myenv".to_string(),
        )));
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_with_confirm_dialog_uninstall_uv() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app();
        app.confirm_dialog = Some(ConfirmDialog::new(ConfirmAction::UninstallUv));
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    // ── draw – create-venv dialog overlay ────────────────────────────────────

    #[test]
    fn test_draw_with_create_dialog() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app();
        app.create_dialog = Some(CreateDialog::new("3.12"));
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_create_dialog_all_fields() {
        use crate::create_field::CreateField;
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        for field in [
            CreateField::Name,
            CreateField::Version,
            CreateField::Packages,
            CreateField::DefaultPkgs,
        ] {
            let mut app = make_app();
            let mut dlg = CreateDialog::new("3.12");
            dlg.field = field;
            dlg.name = "myenv".to_string();
            dlg.version = "3.11".to_string();
            dlg.packages = "requests,flask".to_string();
            dlg.default_pkgs = true;
            app.create_dialog = Some(dlg);
            terminal.draw(|frame| draw(frame, &app)).unwrap();
        }
    }

    // ── status-bar branches ──────────────────────────────────────────────────

    #[test]
    fn test_draw_status_bar_environments_with_venvs() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = make_app_with_venvs();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_status_bar_uv_not_installed() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(vec![], false, None);
        app.next_tab();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    // ── draw – pkg_dialog overlay ─────────────────────────────────────────────

    #[test]
    fn test_draw_with_pkg_dialog_add() {
        use crate::dialogs::{PkgDialog, PkgDialogMode};
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app_with_venvs();
        let mut dlg = PkgDialog::new(PkgDialogMode::Add);
        dlg.input = "requests,flask".to_string();
        app.pkg_dialog = Some(dlg);
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_with_pkg_dialog_remove() {
        use crate::dialogs::{PkgDialog, PkgDialogMode};
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app_with_venvs();
        app.pkg_dialog = Some(PkgDialog::new(PkgDialogMode::Remove));
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    // ── draw – pkg_search mode ────────────────────────────────────────────────

    #[test]
    fn test_draw_with_pkg_search_empty_query() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app_with_venvs();
        app.pkg_search = Some(String::new());
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_with_pkg_search_query() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app_with_venvs();
        app.pkg_search = Some("req".to_string());
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }
    fn assert_help_dialog_renders(tab: Tab) {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app_with_venvs();
        app.tab = tab;
        app.help_dialog = Some(HelpDialog::new(tab.help_mode()));
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }
    #[test]
    fn draw_renders_help_dialog_overlay_for_environments_tab() {
        assert_help_dialog_renders(Tab::Environments);
    }
    #[test]
    fn draw_renders_help_dialog_overlay_for_uv_info_tab() {
        assert_help_dialog_renders(Tab::UvInfo);
    }
}
