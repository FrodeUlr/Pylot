use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
};

use crate::app::{App, Tab};

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

    draw_status_bar(frame, chunks[2]);
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

    let lines = vec![
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
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Astral UV "));

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, area: ratatui::layout::Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" Tab", Style::default().fg(Color::Yellow)),
        Span::raw(": switch tab  "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(": navigate  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(": quit"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help, area);
}
