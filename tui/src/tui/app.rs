use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Terminal,
};
use std::time::{Duration, Instant};

struct App {
    start_time: Instant,
    pulse: f64,
}

impl App {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            pulse: 0.0,
        }
    }

    fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        self.pulse = (elapsed * 2.0).sin() * 0.5 + 0.5;
    }
}

pub fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::new();

    loop {
        app.update();
        terminal.draw(|frame| render(frame, &app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    return Ok(());
                }
            }
        }
    }
}

fn render(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    render_header(frame, chunks[0]);

    // Main content
    render_content(frame, chunks[1], app);

    // Footer
    render_footer(frame, chunks[2]);
}

fn render_header(frame: &mut ratatui::Frame, area: Rect) {
    let title = Paragraph::new("✨ Welcome to My Fancy TUI ✨")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );

    frame.render_widget(title, area);
}

fn render_content(frame: &mut ratatui::Frame, area: Rect, app: &App) {
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
            Span::styled("● Running", Style::default().fg(Color::Green)),
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
        .title(" Animation ")
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

    let anim_widget = Paragraph::new(animation)
        .block(right_block)
        .alignment(Alignment::Center);

    frame.render_widget(anim_widget, chunks[1]);
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " Q ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Quit  "),
        Span::styled(
            " ESC ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Exit  "),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(footer, area);
}
