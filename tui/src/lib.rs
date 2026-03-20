//! TUI module for Pylot
//!
//! Provides a terminal user interface to manage virtual environments and UV.

mod app;
mod ui;

pub use app::App;
use app::{ConfirmAction, ConfirmDialog, CreateDialog, VenvAction};

use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use pylot_shared::constants::DEFAULT_PYTHON_VERSION;
use pylot_shared::uvvenv::UvVenv;
use pylot_shared::venvtraits::{Activate, Create, Delete};
use pylot_shared::{uvctrl, venvmanager};
use std::borrow::Cow;
use std::io;
use std::time::Duration;
use tokio::sync::oneshot;

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
    terminal.clear()?;

    loop {
        run_app(&mut terminal, &mut app).await?;

        // Always restore the TTY to normal mode before doing anything else.
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        log::set_max_level(prev_log_level);

        // The only reason run_app exits is: user quit (no pending action) or Activate.
        let venv_action = app.take_pending_venv_action();
        if venv_action.is_none() {
            break;
        }

        // Handle activate (the one action that must replace the process / spawn a shell).
        if let Some(VenvAction::Activate) = venv_action {
            if !app.venvs.is_empty() {
                let name = app.venvs[app.selected].name.to_string();
                let venv = UvVenv::new(
                    Cow::Owned(name),
                    "".to_string(),
                    "".to_string(),
                    vec![],
                    false,
                );
                match venv.activate().await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error activating venv: {}", e);
                        pause_for_enter();
                    }
                }
            }
        }

        // Refresh state before re-entering the TUI.
        app.uv_installed = uvctrl::check("uv").await.is_ok();
        app.uv_version = if app.uv_installed {
            get_uv_version().await
        } else {
            None
        };
        app.venvs = venvmanager::VENVMANAGER.list().await;
        if !app.venvs.is_empty() && app.selected >= app.venvs.len() {
            app.selected = app.venvs.len() - 1;
        }

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
    let _ = io::stdin().read_line(&mut buf);
}

/// Spawn a background task for a UV management operation and record it in `app`.
fn spawn_uv_task(
    app: &mut App,
    name: &'static str,
    fut: impl std::future::Future<Output = Result<(), String>> + Send + 'static,
) {
    let (tx, rx) = oneshot::channel::<Result<(), String>>();
    tokio::spawn(async move {
        let _ = tx.send(fut.await);
    });
    app.bg_rx = Some(rx);
    app.bg_task_name = Some(name.to_string());
}

/// Spawn a background task for a venv operation and record it in `app`.
fn spawn_venv_task(
    app: &mut App,
    label: String,
    fut: impl std::future::Future<Output = pylot_shared::error::Result<()>> + Send + 'static,
) {
    let (tx, rx) = oneshot::channel::<Result<(), String>>();
    tokio::spawn(async move {
        let _ = tx.send(fut.await.map_err(|e| e.to_string()));
    });
    app.bg_rx = Some(rx);
    app.bg_task_name = Some(label);
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App<'_>,
) -> Result<(), Box<dyn std::error::Error>>
where
    <B as ratatui::backend::Backend>::Error: 'static,
{
    let mut events = EventStream::new();

    loop {
        // --- Poll background task for completion ---
        if let Some(rx) = app.bg_rx.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    app.bg_rx = None;
                    let task_name = app.bg_task_name.take().unwrap_or_default();
                    match result {
                        Ok(()) => {
                            app.status_message =
                                Some((format!("{} completed.", task_name), false));
                        }
                        Err(e) => {
                            app.status_message = Some((format!("Error: {}", e), true));
                        }
                    }
                    // Refresh venv and UV state without leaving the TUI.
                    app.uv_installed = uvctrl::check("uv").await.is_ok();
                    app.uv_version = if app.uv_installed {
                        get_uv_version().await
                    } else {
                        None
                    };
                    app.venvs = venvmanager::VENVMANAGER.list().await;
                    if !app.venvs.is_empty() && app.selected >= app.venvs.len() {
                        app.selected = app.venvs.len() - 1;
                    }
                }
                Err(oneshot::error::TryRecvError::Empty) => {} // still running
                Err(oneshot::error::TryRecvError::Closed) => {
                    app.bg_rx = None;
                    app.bg_task_name = None;
                }
            }
        }

        terminal.draw(|frame| ui::draw(frame, app))?;

        // Wait for a key event or a 200 ms timeout (keeps the spinner ticking).
        let maybe_event = tokio::select! {
            evt = events.next() => evt,
            _ = tokio::time::sleep(Duration::from_millis(200)) => None,
        };

        let Some(Ok(Event::Key(key))) = maybe_event else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        // Any keypress dismisses a one-shot status message.
        app.status_message = None;

        // --- Confirm dialog captures all input while open ---
        if let Some(dialog) = app.confirm_dialog.take() {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    // User confirmed – spawn the appropriate background task.
                    match dialog.action {
                        ConfirmAction::DeleteVenv(name) => {
                            let label = format!("Deleting '{}'", name);
                            spawn_venv_task(app, label, async move {
                                // confirm=false: the confirmation dialog is the prompt.
                                UvVenv::new(
                                    Cow::Owned(name),
                                    "".to_string(),
                                    "".to_string(),
                                    vec![],
                                    false,
                                )
                                .delete(io::Cursor::new(""), false)
                                .await
                            });
                        }
                        ConfirmAction::UninstallUv => {
                            // Pressing 'y' is the user's confirmation – auto-reply "y\n"
                            // so uvctrl::uninstall's stdin prompt is satisfied.
                            spawn_uv_task(
                                app,
                                "Uninstalling UV",
                                uvctrl::uninstall(io::Cursor::new("y\n")),
                            );
                        }
                    }
                }
                // Any other key (including 'n' and Esc) cancels.
                _ => {}
            }
            continue; // dialog consumed the key; skip normal bindings
        }

        // --- Create-venv dialog captures all input while open ---
        if let Some(ref mut dialog) = app.create_dialog {
            match key.code {
                KeyCode::Esc => {
                    app.create_dialog = None;
                }
                KeyCode::Tab | KeyCode::Down => {
                    let next = dialog.field.next();
                    dialog.field = next;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    let prev = dialog.field.prev();
                    dialog.field = prev;
                }
                KeyCode::Char(' ') => {
                    if dialog.field == app::CreateField::DefaultPkgs {
                        dialog.toggle_default();
                    } else {
                        dialog.push_char(' ');
                    }
                }
                KeyCode::Enter => {
                    if dialog.field == app::CreateField::DefaultPkgs {
                        let name = dialog.name.trim().to_string();
                        if !name.is_empty() {
                            let version = dialog.effective_version();
                            let packages = dialog.parsed_packages();
                            let default_pkgs = dialog.default_pkgs;
                            let label = format!("Creating '{}'", name);
                            app.create_dialog = None;
                            // Spawn background task – TUI stays open.
                            spawn_venv_task(app, label, async move {
                                UvVenv::new(
                                    Cow::Owned(name),
                                    "".to_string(),
                                    version,
                                    packages,
                                    default_pkgs,
                                )
                                .create()
                                .await
                            });
                        } else {
                            app.create_dialog = None;
                        }
                    } else {
                        let next = dialog.field.next();
                        dialog.field = next;
                    }
                }
                KeyCode::Backspace => {
                    dialog.pop_char();
                }
                KeyCode::Char(c) => {
                    dialog.push_char(c);
                }
                _ => {}
            }
            continue; // dialog consumed the key; skip normal bindings
        }

        // --- Normal (non-dialog) key bindings ---
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => break,
            KeyCode::Tab | KeyCode::Right => app.next_tab(),
            KeyCode::BackTab | KeyCode::Left => app.prev_tab(),
            KeyCode::Down => app.next_item(),
            KeyCode::Up => app.prev_item(),

            // UV management – only active on the UV Info tab and when not busy.
            KeyCode::Char('i')
                if app.tab == app::Tab::UvInfo
                    && !app.uv_installed
                    && !app.is_busy() =>
            {
                // Pressing 'i' is the user's confirmation – auto-reply "y\n" so
                // uvctrl::install's interactive prompt is satisfied without a shell.
                spawn_uv_task(
                    app,
                    "Installing UV",
                    uvctrl::install(io::Cursor::new("y\n")),
                );
            }
            KeyCode::Char('u')
                if app.tab == app::Tab::UvInfo
                    && app.uv_installed
                    && !app.is_busy() =>
            {
                spawn_uv_task(app, "Updating UV", uvctrl::update());
            }
            KeyCode::Char('d')
                if app.tab == app::Tab::UvInfo
                    && app.uv_installed
                    && !app.is_busy() =>
            {
                // Show a confirmation dialog before uninstalling.
                app.confirm_dialog = Some(ConfirmDialog::new(ConfirmAction::UninstallUv));
            }

            // Venv management – only active on the Environments tab and when not busy.
            KeyCode::Char('n')
                if app.tab == app::Tab::Environments && !app.is_busy() =>
            {
                app.create_dialog = Some(CreateDialog::new(DEFAULT_PYTHON_VERSION));
            }
            KeyCode::Char('d')
                if app.tab == app::Tab::Environments
                    && !app.venvs.is_empty()
                    && !app.is_busy() =>
            {
                let name = app.venvs[app.selected].name.to_string();
                // Show a confirmation dialog before deleting.
                app.confirm_dialog =
                    Some(ConfirmDialog::new(ConfirmAction::DeleteVenv(name)));
            }
            KeyCode::Enter | KeyCode::Char('a')
                if app.tab == app::Tab::Environments
                    && !app.venvs.is_empty()
                    && !app.is_busy() =>
            {
                // Activate still exits the TUI (exec on Unix).
                app.pending_venv_action = Some(VenvAction::Activate);
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn get_uv_version() -> Option<String> {
    use pylot_shared::infra::processes;
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
