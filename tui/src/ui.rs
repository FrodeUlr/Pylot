use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
};

use crate::app::{App, CreateField, Tab};

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
}
fn draw_tabs(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let tab_titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();

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
                    format!("{:<30}", venv.name.as_ref()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("  Python {}", venv.python_version),
                    Style::default().fg(Color::Green),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = format!(" Virtual Environments ({}) ", app.venvs.len());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
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

    frame.render_stateful_widget(list, area, &mut state);
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
                app.uv_version
                    .as_deref()
                    .unwrap_or("N/A"),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Actions:  ", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    if app.uv_installed {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled("[u]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Update    "),
            Span::styled("[d]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Uninstall"),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled("[i]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Install"),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Astral UV "));

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // Priority 1: show a one-shot status message (success or error from a background task).
    if let Some((ref msg, is_error)) = app.status_message {
        let color = if is_error { Color::Red } else { Color::Green };
        let spans = vec![
            Span::styled(
                if is_error { " ✗ " } else { " ✓ " },
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(msg.as_str(), Style::default().fg(color)),
            Span::styled(
                "  (press any key to dismiss)",
                Style::default().fg(Color::DarkGray),
            ),
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
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(task_name.as_str(), Style::default().fg(Color::Cyan)),
            Span::styled("…", Style::default().fg(Color::Yellow)),
            Span::styled("  (q: quit)", Style::default().fg(Color::DarkGray)),
        ];
        let bar = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(bar, area);
        return;
    }

    // When the create dialog is open, show dialog-specific hints instead of the normal bar.
    if app.create_dialog.is_some() {
        let spans = vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": next field  "),
            Span::styled("Shift+Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": prev field  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": confirm  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": cancel"),
        ];
        let help = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(help, area);
        return;
    }

    let mut spans = vec![
        Span::styled(" Tab", Style::default().fg(Color::Yellow)),
        Span::raw(": switch tab  "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(": navigate  "),
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

    spans.push(Span::styled("q", Style::default().fg(Color::Yellow)));
    spans.push(Span::raw(": quit"));

    let help = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(help, area);
}

/// Render the create-venv dialog as a centered overlay popup.
fn draw_create_dialog(frame: &mut Frame, dialog: &crate::app::CreateDialog) {
    let area = centered_rect(60, 14, frame.area());

    // Clear the background so the dialog appears cleanly over other widgets.
    frame.render_widget(Clear, area);

    let focused_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
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

    let lines = vec![
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
            Span::styled(dialog.packages.as_str(), Style::default().fg(Color::Magenta)),
            if dialog.field == CreateField::Packages {
                Span::styled("█", Style::default().fg(Color::Magenta))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(vec![
            Span::raw("                "),
            Span::styled("comma-separated, e.g. requests,flask", hint_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Default pkgs: ", label_style(CreateField::DefaultPkgs)),
            Span::styled(default_indicator, label_style(CreateField::DefaultPkgs)),
            Span::raw("  "),
            Span::styled("(Space to toggle)", hint_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": next field  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": confirm  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" New Virtual Environment ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Yellow)),
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
    Rect { x, y, width: w, height: h }
}
