//! TUI module for Pylot
//!
//! Provides a terminal user interface to manage virtual environments and UV.

mod actions;
mod app;
mod create_dialog;
mod create_field;
mod dialogs;
mod tabs;
mod ui;

use actions::{ConfirmAction, VenvAction};
pub use app::App;
use create_dialog::CreateDialog;
use dialogs::{ConfirmDialog, HelpDialog, PkgDialog, PkgDialogMode};

use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use pylot_shared::constants::DEFAULT_PYTHON_VERSION;
use pylot_shared::uvvenv::UvVenv;
use pylot_shared::venvtraits::{Activate, Create, Delete};
use pylot_shared::{uvctrl, venvmanager};
use ratatui::{backend::CrosstermBackend, Terminal};
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

/// Read the contents of a directory and return sorted entry names.
/// Directory entries get a trailing `/` appended so they can be navigated further.
/// Returns an empty list if the path cannot be read.
fn read_dir_entries_blocking(dir_path: &str) -> Vec<String> {
    let expanded = shellexpand::tilde(dir_path).to_string();
    match std::fs::read_dir(&expanded) {
        Ok(entries) => {
            let mut names: Vec<String> = entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if e.path().is_dir() {
                        format!("{}/", name)
                    } else {
                        name
                    }
                })
                .collect();
            names.sort_by(|a, b| {
                // Directories (trailing '/') sort before files.
                let a_is_dir = a.ends_with('/');
                let b_is_dir = b.ends_with('/');
                b_is_dir
                    .cmp(&a_is_dir)
                    .then(a.to_lowercase().cmp(&b.to_lowercase()))
            });
            names
        }
        Err(_) => Vec::new(),
    }
}

/// Recompute `dialog.completions` based on the current `req_file` value.
///
/// Finds the last `/` in the normalized path and splits it into:
/// * `dir_part`  – everything up to and including the `/`
/// * `filter_prefix` – any text typed after the last `/`
///
/// Reads `dir_part` and filters entries by `filter_prefix` (case-insensitive
/// prefix match). Completions are shown whenever there is at least one `/` in
/// the path, so typing `c:/som` still lists entries in `c:/` that start with
/// `som`.
fn update_completions(dialog: &mut CreateDialog) {
    let normalized = dialog.req_file.replace('\\', "/");

    if let Some(last_slash) = normalized.rfind('/') {
        let dir_part = &normalized[..=last_slash]; // includes trailing '/'
        let filter_prefix = &normalized[last_slash + 1..]; // typed text after '/'

        let mut entries = read_dir_entries_blocking(dir_part);

        // Filter by the text already typed after the last '/'.
        if !filter_prefix.is_empty() {
            let prefix_lower = filter_prefix.to_lowercase();
            entries.retain(|e| {
                // Strip trailing '/' from directory names before comparing.
                e.trim_end_matches('/')
                    .to_lowercase()
                    .starts_with(&prefix_lower)
            });
        }

        if !entries.is_empty() {
            dialog.completions = entries;
            dialog.completions_dir = dir_part.to_string();
            dialog.completion_selected = 0;
            dialog.completion_scroll = 0;
        } else {
            dialog.completions.clear();
            dialog.completions_dir = String::new();
            dialog.completion_selected = 0;
            dialog.completion_scroll = 0;
        }
    } else {
        // No '/' in the path – nothing to complete.
        dialog.completions.clear();
        dialog.completions_dir = String::new();
        dialog.completion_selected = 0;
        dialog.completion_scroll = 0;
    }
}

/// Maximum number of completion entries shown at once in the dialog.
const COMPLETION_MAX_SHOWN: usize = 6;

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
                            app.status_message = Some((format!("{} completed.", task_name), false));
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
            // ── Phase 1: Esc – dismiss completions first, then close dialog ──
            if key.code == KeyCode::Esc {
                if !dialog.completions.is_empty() {
                    dialog.completions.clear();
                    dialog.completion_selected = 0;
                    continue;
                }
                app.create_dialog = None;
                continue;
            }

            // ── Phase 2: Completion navigation (when ReqFile has completions) ──
            let completions_active = dialog.field == create_field::CreateField::ReqFile
                && !dialog.completions.is_empty();
            if completions_active {
                match key.code {
                    KeyCode::Down => {
                        if dialog.completion_selected + 1 < dialog.completions.len() {
                            dialog.completion_selected += 1;
                            // Scroll the window forward if selection goes past visible area.
                            if dialog.completion_selected
                                >= dialog.completion_scroll + COMPLETION_MAX_SHOWN
                            {
                                dialog.completion_scroll += 1;
                            }
                        }
                        continue;
                    }
                    KeyCode::Up => {
                        if dialog.completion_selected > 0 {
                            dialog.completion_selected -= 1;
                            // Scroll the window back if selection goes above visible area.
                            if dialog.completion_selected < dialog.completion_scroll {
                                dialog.completion_scroll =
                                    dialog.completion_scroll.saturating_sub(1);
                            }
                        }
                        continue;
                    }
                    KeyCode::Tab => {
                        // Accept the currently highlighted completion.
                        let sel = dialog.completions[dialog.completion_selected].clone();
                        dialog.req_file_accept_completion(&sel);
                        update_completions(dialog);
                        continue;
                    }
                    _ => {
                        // Any other key: dismiss completions and process normally.
                        dialog.completions.clear();
                        dialog.completion_selected = 0;
                        dialog.completion_scroll = 0;
                        dialog.completions_dir = String::new();
                    }
                }
            }

            // ── Phase 3: Cursor movement (ReqFile field) ──
            if dialog.field == create_field::CreateField::ReqFile {
                match key.code {
                    KeyCode::Left => {
                        dialog.req_file_cursor_left();
                        continue;
                    }
                    KeyCode::Right => {
                        dialog.req_file_cursor_right();
                        continue;
                    }
                    KeyCode::Home => {
                        dialog.req_file_cursor_home();
                        continue;
                    }
                    KeyCode::End => {
                        dialog.req_file_cursor_end();
                        continue;
                    }
                    _ => {}
                }
            }

            // ── Phase 4: Normal field handling ──
            let is_req_file_field = dialog.field == create_field::CreateField::ReqFile;
            match key.code {
                KeyCode::Tab | KeyCode::Down => {
                    let next = dialog.field.next();
                    dialog.field = next;
                    dialog.completions.clear();
                    dialog.completion_selected = 0;
                    dialog.completion_scroll = 0;
                    dialog.completions_dir = String::new();
                }
                KeyCode::BackTab | KeyCode::Up => {
                    let prev = dialog.field.prev();
                    dialog.field = prev;
                    dialog.completions.clear();
                    dialog.completion_selected = 0;
                    dialog.completion_scroll = 0;
                    dialog.completions_dir = String::new();
                }
                KeyCode::Char(' ') => {
                    if dialog.field == create_field::CreateField::DefaultPkgs {
                        dialog.toggle_default();
                    } else {
                        dialog.push_char(' ');
                    }
                }
                KeyCode::Enter => {
                    if dialog.field == create_field::CreateField::DefaultPkgs {
                        let name = dialog.name.trim().to_string();
                        if !name.is_empty() {
                            let version = dialog.effective_version();
                            let packages = dialog.parsed_packages();
                            let default_pkgs = dialog.default_pkgs;
                            // Normalize Windows paths before storing.
                            let req_file = dialog.req_file.trim().replace('\\', "/");
                            let req_file_opt = if req_file.is_empty() {
                                None
                            } else {
                                Some(req_file)
                            };
                            let label = format!("Creating '{}'", name);
                            app.create_dialog = None;
                            // Spawn background task – TUI stays open.
                            spawn_venv_task(app, label, async move {
                                let venv = UvVenv::new(
                                    Cow::Owned(name),
                                    "".to_string(),
                                    version,
                                    packages,
                                    default_pkgs,
                                );
                                venv.create().await?;
                                if let Some(ref path) = req_file_opt {
                                    venv.install_from_requirements(path).await.map_err(|e| {
                                        pylot_shared::error::PylotError::Other(format!(
                                            "Venv created; requirements install failed: {}",
                                            e
                                        ))
                                    })?;
                                }
                                Ok(())
                            });
                            continue; // dialog is consumed; skip Phase 5
                        } else {
                            app.create_dialog = None;
                            continue; // dialog is consumed; skip Phase 5
                        }
                    } else {
                        let next = dialog.field.next();
                        dialog.field = next;
                        dialog.completions.clear();
                        dialog.completion_selected = 0;
                        dialog.completion_scroll = 0;
                        dialog.completions_dir = String::new();
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

            // ── Phase 5: Recompute completions after any change to req_file ──
            if is_req_file_field {
                update_completions(dialog);
            }

            continue; // dialog consumed the key; skip normal bindings
        }

        // --- Package-management dialog captures all input while open ---
        if app.pkg_dialog.is_some() {
            match key.code {
                KeyCode::Esc => {
                    app.pkg_dialog = None;
                }
                KeyCode::Enter => {
                    if let Some(dialog) = app.pkg_dialog.take() {
                        let packages = dialog.parsed_packages();
                        if !packages.is_empty() && !app.venvs.is_empty() {
                            let name = app.venvs[app.selected].name.to_string();
                            match dialog.mode {
                                PkgDialogMode::Add => {
                                    let label = format!("Adding packages to '{}'", name);
                                    spawn_venv_task(app, label, async move {
                                        UvVenv::new(
                                            Cow::Owned(name),
                                            "".to_string(),
                                            "".to_string(),
                                            vec![],
                                            false,
                                        )
                                        .add_packages(packages)
                                        .await
                                    });
                                }
                                PkgDialogMode::Remove => {
                                    let label = format!("Removing packages from '{}'", name);
                                    spawn_venv_task(app, label, async move {
                                        UvVenv::new(
                                            Cow::Owned(name),
                                            "".to_string(),
                                            "".to_string(),
                                            vec![],
                                            false,
                                        )
                                        .remove_packages(packages)
                                        .await
                                    });
                                }
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    if let Some(ref mut d) = app.pkg_dialog {
                        d.pop_char();
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(ref mut d) = app.pkg_dialog {
                        d.push_char(c);
                    }
                }
                _ => {}
            }
            continue; // dialog consumed the key; skip normal bindings
        }

        // --- Package search mode captures all input while active ---
        if app.pkg_search.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    app.pkg_search = None;
                }
                KeyCode::Backspace => {
                    if let Some(ref mut q) = app.pkg_search {
                        q.pop();
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(ref mut q) = app.pkg_search {
                        q.push(c);
                    }
                }
                _ => {}
            }
            continue; // search mode consumed the key; skip normal bindings
        }

        // --- Help dialog captures all input while open ---
        if app.help_dialog.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.help_dialog = None;
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
                if app.tab == tabs::Tab::UvInfo && !app.uv_installed && !app.is_busy() =>
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
                if app.tab == tabs::Tab::UvInfo && app.uv_installed && !app.is_busy() =>
            {
                spawn_uv_task(app, "Updating UV", uvctrl::update());
            }
            KeyCode::Char('d')
                if app.tab == tabs::Tab::UvInfo && app.uv_installed && !app.is_busy() =>
            {
                // Show a confirmation dialog before uninstalling.
                app.confirm_dialog = Some(ConfirmDialog::new(ConfirmAction::UninstallUv));
            }

            // Venv management – only active on the Environments tab and when not busy.
            KeyCode::Char('n') if app.tab == tabs::Tab::Environments && !app.is_busy() => {
                app.create_dialog = Some(CreateDialog::new(DEFAULT_PYTHON_VERSION));
            }
            KeyCode::Char('d')
                if app.tab == tabs::Tab::Environments
                    && !app.venvs.is_empty()
                    && !app.is_busy() =>
            {
                let name = app.venvs[app.selected].name.to_string();
                // Show a confirmation dialog before deleting.
                app.confirm_dialog = Some(ConfirmDialog::new(ConfirmAction::DeleteVenv(name)));
            }
            KeyCode::Char('?') => {
                app.help_dialog = Some(HelpDialog::new(app.tab.help_mode()));
            }
            KeyCode::Enter
                if app.tab == tabs::Tab::Environments
                    && !app.venvs.is_empty()
                    && !app.is_busy() =>
            {
                // Activate still exits the TUI (exec on Unix).
                app.pending_venv_action = Some(VenvAction::Activate);
                break;
            }
            // Package list scrolling – active when a venv is selected.
            KeyCode::Char('j') if app.tab == tabs::Tab::Environments && !app.venvs.is_empty() => {
                let total = app.venvs[app.selected].installed_packages.len();
                app.scroll_pkg_down(total);
            }
            KeyCode::Char('k') if app.tab == tabs::Tab::Environments && !app.venvs.is_empty() => {
                app.scroll_pkg_up();
            }
            // Add packages – active when a venv is selected and not busy.
            KeyCode::Char('i') | KeyCode::Char('a')
                if app.tab == tabs::Tab::Environments
                    && !app.venvs.is_empty()
                    && !app.is_busy() =>
            {
                app.pkg_dialog = Some(PkgDialog::new(PkgDialogMode::Add));
            }
            // Remove packages – active when a venv is selected and not busy.
            KeyCode::Char('r')
                if app.tab == tabs::Tab::Environments
                    && !app.venvs.is_empty()
                    && !app.is_busy() =>
            {
                app.pkg_dialog = Some(PkgDialog::new(PkgDialogMode::Remove));
            }
            // Search packages – active when a venv is selected.
            KeyCode::Char('/') if app.tab == tabs::Tab::Environments && !app.venvs.is_empty() => {
                app.pkg_search = Some(String::new());
                app.pkg_scroll = 0;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    fn make_empty_app<'a>() -> App<'a> {
        App::new(vec![], true, Some("uv 0.5.0".to_string()))
    }

    // ── get_uv_version ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_uv_version_returns_some_or_none() {
        // We don't know whether `uv` is installed in the test environment, so
        // just verify that the function returns without panicking.
        let _version = get_uv_version().await;
    }

    // ── spawn_uv_task ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_spawn_uv_task_sets_busy() {
        let mut app = make_empty_app();
        assert!(!app.is_busy());
        spawn_uv_task(&mut app, "Test task", async { Ok(()) });
        assert!(app.is_busy());
        assert_eq!(app.bg_task_name.as_deref(), Some("Test task"));
    }

    // ── spawn_venv_task ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_spawn_venv_task_sets_busy() {
        let mut app = make_empty_app();
        assert!(!app.is_busy());
        spawn_venv_task(&mut app, "Venv task".to_string(), async {
            Ok::<(), pylot_shared::error::PylotError>(())
        });
        assert!(app.is_busy());
        assert_eq!(app.bg_task_name.as_deref(), Some("Venv task"));
    }

    // ── pause_for_enter is a side-effectful helper; just verify it compiles.  ─
    // (It reads from stdin which we can't easily mock in a unit test, so we
    //  skip calling it directly and rely on the compiler for basic coverage.)

    // ── read_dir_entries_blocking ────────────────────────────────────────────

    #[test]
    fn test_read_dir_entries_blocking_nonexistent() {
        // A path that doesn't exist → empty list without panicking.
        let result = read_dir_entries_blocking("/nonexistent_path_xyz_abc_123456");
        assert!(result.is_empty());
    }

    #[test]
    fn test_read_dir_entries_blocking_valid_dir() {
        // Any readable directory should not panic and return a Vec.
        // We use the system temp dir which is guaranteed to exist.
        let tmp = std::env::temp_dir();
        let path = tmp.to_str().unwrap();
        let _result = read_dir_entries_blocking(path);
        // Just verify it does not panic; content is environment-dependent.
    }

    #[test]
    fn test_read_dir_entries_blocking_dirs_before_files() {
        use std::fs;
        let dir = tempfile::TempDir::new().unwrap();

        // Create a file and a subdirectory; the directory should sort first.
        fs::write(dir.path().join("b_file.txt"), "content").unwrap();
        fs::create_dir(dir.path().join("a_subdir")).unwrap();

        let path = dir.path().to_str().unwrap();
        let entries = read_dir_entries_blocking(path);

        assert!(!entries.is_empty(), "Expected at least one entry");
        // The first entry must be a directory (trailing '/').
        assert!(
            entries[0].ends_with('/'),
            "Expected first entry to be a directory, got '{}'",
            entries[0]
        );
    }

    #[test]
    fn test_read_dir_entries_blocking_file_no_trailing_slash() {
        use std::fs;
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join("myfile.txt"), "content").unwrap();

        let path = dir.path().to_str().unwrap();
        let entries = read_dir_entries_blocking(path);

        let file_entry = entries.iter().find(|e| e.starts_with("myfile"));
        assert!(file_entry.is_some(), "Expected myfile.txt in entries");
        assert!(
            !file_entry.unwrap().ends_with('/'),
            "File entry should not have trailing '/'"
        );
    }

    #[test]
    fn test_read_dir_entries_blocking_dir_has_trailing_slash() {
        use std::fs;
        let dir = tempfile::TempDir::new().unwrap();
        fs::create_dir(dir.path().join("mysubdir")).unwrap();

        let path = dir.path().to_str().unwrap();
        let entries = read_dir_entries_blocking(path);

        let dir_entry = entries.iter().find(|e| e.starts_with("mysubdir"));
        assert!(dir_entry.is_some(), "Expected mysubdir in entries");
        assert_eq!(
            dir_entry.unwrap().as_str(),
            "mysubdir/",
            "Directory entry should have trailing '/'"
        );
    }

    #[test]
    fn test_read_dir_entries_blocking_tilde_expansion() {
        // Tilde should be expanded without panicking; result may be empty
        // if the home directory is inaccessible.
        let _result = read_dir_entries_blocking("~/");
    }

    // ── update_completions ───────────────────────────────────────────────────

    #[test]
    fn test_update_completions_no_slash_clears_completions() {
        let mut dialog = CreateDialog::new("3.12");
        dialog.req_file = "justtext".to_string();
        // Pre-populate completions to verify they are cleared.
        dialog.completions = vec!["something/".to_string()];
        dialog.completions_dir = "dir/".to_string();
        dialog.completion_selected = 1;
        dialog.completion_scroll = 2;

        update_completions(&mut dialog);

        assert!(dialog.completions.is_empty());
        assert!(dialog.completions_dir.is_empty());
        assert_eq!(dialog.completion_selected, 0);
        assert_eq!(dialog.completion_scroll, 0);
    }

    #[test]
    fn test_update_completions_empty_req_file_clears() {
        let mut dialog = CreateDialog::new("3.12");
        dialog.req_file = String::new();
        dialog.completions = vec!["old/".to_string()];

        update_completions(&mut dialog);

        assert!(dialog.completions.is_empty());
    }

    #[test]
    fn test_update_completions_backslash_normalized_to_slash() {
        // Backslash paths normalise to '/' before the rfind check.
        // Use a path whose directory portion does not exist → completions cleared.
        let mut dialog = CreateDialog::new("3.12");
        dialog.req_file = "C:\\nonexistent_xyz_abc\\".to_string();

        update_completions(&mut dialog);

        // Non-existent directory → no entries → completions cleared.
        assert!(dialog.completions.is_empty());
    }

    #[test]
    fn test_update_completions_valid_dir_populates_completions() {
        use std::fs;
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "numpy\n").unwrap();

        // Normalise to forward slashes so the expected value matches what
        // `update_completions` stores in `completions_dir` after its own
        // backslash → forward-slash normalisation (important on Windows).
        let dir_path = dir.path().to_str().unwrap().replace('\\', "/");
        let mut dialog = CreateDialog::new("3.12");
        // Trailing '/' makes filter_prefix empty → list all entries.
        dialog.req_file = format!("{}/", dir_path);

        update_completions(&mut dialog);

        assert!(
            !dialog.completions.is_empty(),
            "Expected completions to be populated"
        );
        assert_eq!(dialog.completions_dir, format!("{}/", dir_path));
        assert_eq!(dialog.completion_selected, 0);
        assert_eq!(dialog.completion_scroll, 0);
    }

    #[test]
    fn test_update_completions_filter_prefix_narrows_results() {
        use std::fs;
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "numpy\n").unwrap();
        fs::write(dir.path().join("other.txt"), "scipy\n").unwrap();

        let dir_path = dir.path().to_str().unwrap();
        let mut dialog = CreateDialog::new("3.12");
        // Filter prefix "req" should only match "requirements.txt".
        dialog.req_file = format!("{}/req", dir_path);

        update_completions(&mut dialog);

        assert!(!dialog.completions.is_empty());
        for entry in &dialog.completions {
            assert!(
                entry
                    .trim_end_matches('/')
                    .to_lowercase()
                    .starts_with("req"),
                "Completion '{}' should start with 'req'",
                entry
            );
        }
    }

    #[test]
    fn test_update_completions_no_match_clears() {
        use std::fs;
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "numpy\n").unwrap();

        let dir_path = dir.path().to_str().unwrap();
        let mut dialog = CreateDialog::new("3.12");
        // Pre-populate completions to verify they get cleared.
        dialog.completions = vec!["old/".to_string()];
        dialog.completion_selected = 1;
        // Filter prefix that matches nothing.
        dialog.req_file = format!("{}/zzz_no_match_xyz", dir_path);

        update_completions(&mut dialog);

        assert!(dialog.completions.is_empty());
        assert!(dialog.completions_dir.is_empty());
        assert_eq!(dialog.completion_selected, 0);
        assert_eq!(dialog.completion_scroll, 0);
    }

    #[test]
    fn test_update_completions_nonexistent_dir_clears() {
        let mut dialog = CreateDialog::new("3.12");
        // Valid slash present but the directory doesn't exist.
        dialog.req_file = "/nonexistent_xyz_abc_123/somefile".to_string();
        dialog.completions = vec!["old/".to_string()];

        update_completions(&mut dialog);

        assert!(dialog.completions.is_empty());
        assert!(dialog.completions_dir.is_empty());
    }

    // ── COMPLETION_MAX_SHOWN constant ────────────────────────────────────────

    #[test]
    fn test_completion_max_shown_is_six() {
        assert_eq!(COMPLETION_MAX_SHOWN, 6);
    }
}
