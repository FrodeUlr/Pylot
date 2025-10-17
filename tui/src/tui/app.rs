use super::content::render_content;
use super::header_footer::{render_footer, render_header};
use ratatui::layout::{Constraint, Direction, Layout};
use std::time::Instant;

pub struct App {
    pub start_time: Instant,
    pub pulse: f64,
}

impl App {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            pulse: 0.0,
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        self.pulse = (elapsed * 2.0).sin() * 0.5 + 0.5;
    }
}

pub fn render<'a>(frame: &mut ratatui::Frame<'a>, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(frame, chunks[0]);

    render_content(frame, chunks[1], app);

    render_footer(frame, chunks[2]);
}
