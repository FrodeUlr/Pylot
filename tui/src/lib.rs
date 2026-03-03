//! TUI module for Pylot
//!
//! Provides a terminal user interface to manage virtual environments and UV.

mod app;
mod ui;

pub use app::App;
use app::UvAction;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
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

    // Suppress all log output while the TUI is active so that mio/tokio trace
    // messages (and any other log output) cannot write to the TTY and corrupt
    // the alternate-screen display.
    let prev_log_level = log::max_level();
    log::set_max_level(log::LevelFilter::Off);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Clear the alternate screen so no stale content remains before the first
    // draw, and ratatui's diff works from a known-clean baseline.
    terminal.clear()?;

    loop {
        let result = run_app(&mut terminal, &mut app);

        // Always restore the TTY to normal mode before doing anything else.
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        log::set_max_level(prev_log_level);

        // Propagate terminal errors immediately.
        result?;

        match app.take_pending_action() {
            // User pressed q/Esc – we are done.
            None => break,

            // User triggered a UV management action. Execute it in normal
            // terminal mode, then re-enter the TUI.
            Some(action) => {
                match action {
                    UvAction::Install => {
                        if let Err(e) = uvctrl::install(io::stdin()).await {
                            log::error!("{}", e);
                        }
                    }
                    UvAction::Update => {
                        if let Err(e) = uvctrl::update().await {
                            log::error!("{}", e);
                        }
                    }
                    UvAction::Uninstall => {
                        if let Err(e) = uvctrl::uninstall(io::stdin()).await {
                            log::error!("{}", e);
                        }
                    }
                }

                println!("\nPress Enter to return to TUI...");
                let mut buf = String::new();
                // Ignore read errors (e.g. non-interactive stdin) and continue.
                let _ = io::stdin().read_line(&mut buf);

                // Refresh UV state after the action.
                app.uv_installed = uvctrl::check("uv").await.is_ok();
                app.uv_version = if app.uv_installed {
                    get_uv_version().await
                } else {
                    None
                };
                app.venvs = venvmanager::VENVMANAGER.list().await;

                // Re-enter the TUI.
                log::set_max_level(log::LevelFilter::Off);
                enable_raw_mode()?;
                let mut stdout = io::stdout();
                execute!(stdout, EnterAlternateScreen)?;
                let backend = CrosstermBackend::new(stdout);
                terminal = Terminal::new(backend)?;
                terminal.clear()?;
            }
        }
    }

    Ok(())
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
                // UV management actions – only active on the UV Info tab.
                KeyCode::Char('i') if app.tab == app::Tab::UvInfo && !app.uv_installed => {
                    app.pending_action = Some(UvAction::Install);
                    break;
                }
                KeyCode::Char('u') if app.tab == app::Tab::UvInfo && app.uv_installed => {
                    app.pending_action = Some(UvAction::Update);
                    break;
                }
                KeyCode::Char('d') if app.tab == app::Tab::UvInfo && app.uv_installed => {
                    app.pending_action = Some(UvAction::Uninstall);
                    break;
                }
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
