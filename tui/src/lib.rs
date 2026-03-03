//! TUI module for Pylot
//!
//! Provides a terminal user interface to manage virtual environments and UV.

mod app;
mod ui;

pub use app::App;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use shared::{uvctrl, venvmanager};
use std::io;

/// Run the TUI application
///
/// # Returns
/// * `Result<()>` - Ok if the TUI ran successfully
///
/// # Examples
/// ```no_run
/// use pylot_tui::run;
/// run();
/// ```
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let venvs = venvmanager::VENVMANAGER.list().await;
    let uv_installed = uvctrl::check("uv").await.is_ok();
    let uv_version = if uv_installed {
        get_uv_version().await
    } else {
        None
    };

    let mut app = App::new(venvs, uv_installed, uv_version);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>>
where
    <B as ratatui::backend::Backend>::Error: 'static,
{
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Tab | KeyCode::Right => app.next_tab(),
                KeyCode::BackTab | KeyCode::Left => app.prev_tab(),
                KeyCode::Down => app.next_item(),
                KeyCode::Up => app.prev_item(),
                _ => {}
            }
        }
    }
    Ok(())
}

async fn get_uv_version() -> Option<String> {
    use shared::core::processes;
    let child = processes::create_child_cmd("uv", &["version"], "").ok()?;
    let output = child.wait_with_output().await.ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    } else {
        None
    }
}
