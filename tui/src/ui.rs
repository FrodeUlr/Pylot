use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
};

use crate::app::{App, ConfirmDialog, CreateField, Tab};

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
    // Split horizontally: venv list (left) + detail panel (right)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(area);

    // ── Left: venv list ──────────────────────────────────────────────────────
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
                    format!("  py{}", venv.python_version),
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

    frame.render_stateful_widget(list, columns[0], &mut state);

    // ── Right: detail panel ──────────────────────────────────────────────────
    draw_venv_detail(frame, app, columns[1]);
}

fn draw_venv_detail(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let label_style = Style::default().fg(Color::DarkGray);
    let value_style = Style::default().fg(Color::White);

    let lines = if app.venvs.is_empty() {
        vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No environments found.",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Press [n] to create one.",
                Style::default().fg(Color::DarkGray),
            )]),
        ]
    } else {
        let venv = &app.venvs[app.selected];

        // Replace the home directory prefix with ~ for a compact display.
        let display_path = pylot_shared::utils::shorten_home_path(venv.path.as_str());

        let pkg_text = match venv.package_count {
            Some(n) => format!("{} installed", n),
            None => "unknown".to_string(),
        };

        let version_text = if venv.python_version.is_empty() {
            "unknown".to_string()
        } else {
            format!("Python {}", venv.python_version)
        };

        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Name     : ", label_style),
                Span::styled(
                    venv.name.as_ref().to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Python   : ", label_style),
                Span::styled(version_text, Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Location : ", label_style),
                Span::styled(display_path, value_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Packages : ", label_style),
                Span::styled(pkg_text, Style::default().fg(Color::Magenta)),
            ]),
        ]
    };

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Details "))
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(paragraph, area);
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

    // Priority 3: show confirm-dialog hints when a confirmation is pending.
    if app.confirm_dialog.is_some() {
        let spans = vec![
            Span::styled("y / Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(": confirm  "),
            Span::styled("n / Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(": cancel"),
        ];
        let help = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(help, area);
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
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("This action cannot be undone.", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[y] Yes", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("    "),
            Span::styled("[n] No", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        App, ConfirmAction, ConfirmDialog, CreateDialog,
    };
    use pylot_shared::uvvenv::UvVenv;
    use ratatui::{Terminal, backend::TestBackend};
    use std::borrow::Cow;

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
        let r = Rect { x: 0, y: 0, width: 100, height: 50 };
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
        let r = Rect { x: 0, y: 0, width: 40, height: 10 };
        let result = centered_rect(100, 50, r);
        // Should be clamped to the parent's dimensions.
        assert_eq!(result.width, 40);
        assert_eq!(result.height, 10);
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
    }

    #[test]
    fn test_centered_rect_zero_size() {
        let r = Rect { x: 5, y: 3, width: 80, height: 24 };
        let result = centered_rect(0, 0, r);
        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    #[test]
    fn test_centered_rect_offset_parent() {
        let r = Rect { x: 10, y: 5, width: 80, height: 24 };
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

    // ── draw – status messages ───────────────────────────────────────────────

    #[test]
    fn test_draw_with_status_message_ok() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app();
        app.status_message = Some(("Operation completed.".to_string(), false));
        terminal.draw(|frame| draw(frame, &app)).unwrap();
    }

    #[test]
    fn test_draw_with_status_message_error() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = make_app();
        app.status_message = Some(("Something went wrong.".to_string(), true));
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
        use crate::app::CreateField;
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
}
