use std::sync::{LazyLock, Mutex};

use super::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

static STATUS_MESSAGE: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::from("Running")));

pub fn set_status_message(status: &str) {
    let mut msg = STATUS_MESSAGE.lock().unwrap();
    *msg = status.to_string();
}

fn get_status_message() -> String {
    let msg = STATUS_MESSAGE.lock().unwrap();
    msg.clone()
}

pub fn render_content(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left panel
    let left_block = Block::default()
        .title(" Info ")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .padding(Padding::uniform(2))
        .style(Style::default().bg(Color::Black));

    let elapsed = app.start_time.elapsed().as_secs();
    let info_text = vec![
        Line::from(vec![
            Span::styled(
                "Hello ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "World!",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Runtime: "),
            Span::styled(format!("{}s", elapsed), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(get_status_message(), Style::default().fg(Color::Green)),
        ]),
    ];

    let info = Paragraph::new(info_text)
        .block(left_block)
        .alignment(Alignment::Left);

    frame.render_widget(info, chunks[0]);

    // Right panel with pulsing effect
    let pulse_intensity = (app.pulse * 255.0) as u8;
    let pulse_color = Color::Rgb(pulse_intensity, 100, 255 - pulse_intensity);
    let right_block = Block::default()
        .title(" Venvs ")
        .title_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(pulse_color))
        .padding(Padding::uniform(2))
        .style(Style::default().bg(Color::Black));

    let animation = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("        "),
            Span::styled(
                "◆",
                Style::default()
                    .fg(pulse_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("       "),
            Span::styled("◆ ◆", Style::default().fg(pulse_color)),
        ]),
        Line::from(vec![
            Span::raw("      "),
            Span::styled("◆ ◆ ◆", Style::default().fg(pulse_color)),
        ]),
        Line::from(vec![
            Span::raw("       "),
            Span::styled("◆ ◆", Style::default().fg(pulse_color)),
        ]),
        Line::from(vec![
            Span::raw("        "),
            Span::styled(
                "◆",
                Style::default()
                    .fg(pulse_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let venvs = shared::venvmanager::VENVMANAGER.list_sync();

    let venvs_text = venvs
        .iter()
        .map(|v| format!("{} - {}", v.name, v.path))
        .collect::<Vec<_>>()
        .join("\n");
    let anim_widget = Paragraph::new(venvs_text)
        .block(right_block)
        .alignment(Alignment::Center);
    frame.render_widget(anim_widget, chunks[1]);
}
