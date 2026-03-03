//! TUI module for Pylot
//!
//! Provides a terminal user interface to manage virtual environments and UV.

mod app;
mod ui;

pub use app::App;
use app::{UvAction, VenvAction};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use shared::constants::DEFAULT_PYTHON_VERSION;
use shared::uvvenv::UvVenv;
use shared::venvtraits::{Activate, Create, Delete};
use shared::{uvctrl, venvmanager};
use std::borrow::Cow;
use std::io::{self, Write};

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

        let uv_action = app.take_pending_action();
        let venv_action = app.take_pending_venv_action();

        // No pending action means the user quit.
        if uv_action.is_none() && venv_action.is_none() {
            break;
        }

        // --- UV management actions ---
        if let Some(action) = uv_action {
            match action {
                UvAction::Install => {
                    if let Err(e) = uvctrl::install(io::stdin()).await {
                        eprintln!("Error: {}", e);
                    }
                }
                UvAction::Update => {
                    if let Err(e) = uvctrl::update().await {
                        eprintln!("Error: {}", e);
                    }
                }
                UvAction::Uninstall => {
                    if let Err(e) = uvctrl::uninstall(io::stdin()).await {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            pause_for_enter();
        }

        // --- Venv management actions ---
        if let Some(action) = venv_action {
            match action {
                VenvAction::Create => {
                    if let Some((name, version, pkgs, default)) = prompt_create_venv().await {
                        // The direct `path` field is unused by `create()`; the actual
                        // path is resolved from `settings.venvs_path` + `name`.
                        let venv = UvVenv::new(
                            Cow::Owned(name),
                            "".to_string(),
                            version,
                            pkgs,
                            default,
                        );
                        match venv.create().await {
                            Ok(_) => println!("Virtual environment created successfully."),
                            Err(e) => eprintln!("Error creating venv: {}", e),
                        }
                    }
                    pause_for_enter();
                }
                VenvAction::Delete => {
                    if !app.venvs.is_empty() {
                        let name = app.venvs[app.selected].name.to_string();
                        // The direct `path` field is unused by `delete()`; path is
                        // resolved from `settings.venvs_path` + `name`.
                        let venv = UvVenv::new(
                            Cow::Owned(name),
                            "".to_string(),
                            "".to_string(),
                            vec![],
                            false,
                        );
                        if let Err(e) = venv.delete(io::stdin(), true).await {
                            eprintln!("Error deleting venv: {}", e);
                        }
                    }
                    pause_for_enter();
                }
                VenvAction::Activate => {
                    if !app.venvs.is_empty() {
                        let name = app.venvs[app.selected].name.to_string();
                        // The direct `path` field is unused by `activate()`; path is
                        // resolved from `settings.venvs_path` + `name`.
                        let venv = UvVenv::new(
                            Cow::Owned(name),
                            "".to_string(),
                            "".to_string(),
                            vec![],
                            false,
                        );
                        match venv.activate().await {
                            // On Windows the child shell exited; re-enter TUI below.
                            Ok(_) => {}
                            // On Unix exec() never returns on success, so Err means
                            // activation failed. Show the error and re-enter TUI.
                            Err(e) => {
                                eprintln!("Error activating venv: {}", e);
                                pause_for_enter();
                            }
                        }
                    }
                }
            }
        }

        // Refresh UV + venv state before re-entering the TUI.
        app.uv_installed = uvctrl::check("uv").await.is_ok();
        app.uv_version = if app.uv_installed {
            get_uv_version().await
        } else {
            None
        };
        app.venvs = venvmanager::VENVMANAGER.list().await;
        // Clamp the cursor in case the list shrank (e.g. after a delete).
        if !app.venvs.is_empty() && app.selected >= app.venvs.len() {
            app.selected = app.venvs.len() - 1;
        }

        // Re-enter the TUI.
        log::set_max_level(log::LevelFilter::Off);
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        terminal = Terminal::new(backend)?;
        terminal.clear()?;
    }

    Ok(())
}

fn pause_for_enter() {
    println!("\nPress Enter to return to TUI...");
    let mut buf = String::new();
    // Ignore read errors (e.g. non-interactive stdin) and continue.
    let _ = io::stdin().read_line(&mut buf);
}

/// Prompt the user for new venv details and return them, or `None` if cancelled.
async fn prompt_create_venv() -> Option<(String, String, Vec<String>, bool)> {
    println!("\n--- Create Virtual Environment ---");

    print!("Name: ");
    io::stdout().flush().ok();
    let mut name = String::new();
    io::stdin().read_line(&mut name).ok();
    let name = name.trim().to_string();
    if name.is_empty() {
        println!("Cancelled (no name provided).");
        return None;
    }

    print!("Python version [{}]: ", DEFAULT_PYTHON_VERSION);
    io::stdout().flush().ok();
    let mut version = String::new();
    io::stdin().read_line(&mut version).ok();
    let version = {
        let v = version.trim();
        if v.is_empty() {
            DEFAULT_PYTHON_VERSION.to_string()
        } else {
            v.to_string()
        }
    };

    print!("Packages (comma-separated, empty=none): ");
    io::stdout().flush().ok();
    let mut pkgs_input = String::new();
    io::stdin().read_line(&mut pkgs_input).ok();
    let pkgs: Vec<String> = pkgs_input
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    print!("Install default packages? [y/N]: ");
    io::stdout().flush().ok();
    let mut default_input = String::new();
    io::stdin().read_line(&mut default_input).ok();
    let default_pkgs = {
        let t = default_input.trim();
        t.eq_ignore_ascii_case("y") || t.eq_ignore_ascii_case("yes")
    };

    Some((name, version, pkgs, default_pkgs))
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
                // Venv management actions – only active on the Environments tab.
                KeyCode::Char('n') if app.tab == app::Tab::Environments => {
                    app.pending_venv_action = Some(VenvAction::Create);
                    break;
                }
                KeyCode::Char('d')
                    if app.tab == app::Tab::Environments && !app.venvs.is_empty() =>
                {
                    app.pending_venv_action = Some(VenvAction::Delete);
                    break;
                }
                KeyCode::Enter | KeyCode::Char('a')
                    if app.tab == app::Tab::Environments && !app.venvs.is_empty() =>
                {
                    app.pending_venv_action = Some(VenvAction::Activate);
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
