use std::time::Instant;

use pylot_shared::virtualenv::uvvenv::UvVenv;

use crate::actions::{UvAction, VenvAction};
use crate::create_dialog::CreateDialog;
use crate::dialogs::{ConfirmDialog, HelpDialog, PkgDialog};
use crate::tabs::Tab;

/// How long (in seconds) a one-shot status message is shown before auto-dismissal.
pub const STATUS_MESSAGE_TIMEOUT_SECS: u64 = 3;

/// Application state for the TUI
pub struct App<'a> {
    pub tab: Tab,
    pub venvs: Vec<UvVenv<'a>>,
    pub selected: usize,
    pub uv_installed: bool,
    pub uv_version: Option<String>,
    /// Latest UV version available, used to show an update indicator.
    pub uv_latest_version: Option<String>,
    /// Receiver end of the background UV info fetch (version + latest).
    pub uv_info_rx:
        Option<tokio::sync::oneshot::Receiver<(Option<String>, Option<String>)>>,
    pub pending_action: Option<UvAction>,
    pub pending_venv_action: Option<VenvAction>,
    /// When `Some`, the create-venv dialog is open.
    pub create_dialog: Option<CreateDialog>,
    /// When `Some`, a yes/no confirmation overlay is open.
    pub confirm_dialog: Option<ConfirmDialog>,
    pub help_dialog: Option<HelpDialog>,
    /// Receiver end of the channel used to collect a background task's result.
    pub bg_rx: Option<tokio::sync::oneshot::Receiver<Result<(), String>>>,
    /// Human-readable label for the running task, shown in the status bar.
    pub bg_task_name: Option<String>,
    /// One-shot status message (text, is_error, set_at) – cleared after 3 s or any keypress.
    pub status_message: Option<(String, bool, Instant)>,
    /// When `Some`, the add/remove-package dialog is open.
    pub pkg_dialog: Option<PkgDialog>,
    /// When `Some`, package search is active with this query string.
    pub pkg_search: Option<String>,
    /// Scroll offset for the packages list in the detail panel.
    pub pkg_scroll: usize,
}

impl<'a> App<'a> {
    pub fn new(venvs: Vec<UvVenv<'a>>, uv_installed: bool, uv_version: Option<String>) -> Self {
        App {
            tab: Tab::Environments,
            venvs,
            selected: 0,
            uv_installed,
            uv_version,
            uv_latest_version: None,
            uv_info_rx: None,
            pending_action: None,
            pending_venv_action: None,
            create_dialog: None,
            confirm_dialog: None,
            help_dialog: None,
            pkg_dialog: None,
            pkg_search: None,
            bg_rx: None,
            bg_task_name: None,
            status_message: None,
            pkg_scroll: 0,
        }
    }

    pub fn next_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Environments => Tab::UvInfo,
            Tab::UvInfo => Tab::Environments,
        };
        self.selected = 0;
    }

    pub fn prev_tab(&mut self) {
        self.next_tab();
    }

    pub fn next_item(&mut self) {
        if self.tab == Tab::Environments && !self.venvs.is_empty() {
            self.selected = (self.selected + 1) % self.venvs.len();
            self.pkg_scroll = 0;
        }
    }

    pub fn prev_item(&mut self) {
        if self.tab == Tab::Environments && !self.venvs.is_empty() {
            if self.selected == 0 {
                self.selected = self.venvs.len() - 1;
            } else {
                self.selected -= 1;
            }
            self.pkg_scroll = 0;
        }
    }

    /// Take (remove and return) a pending UV action, if any.
    pub fn take_pending_action(&mut self) -> Option<UvAction> {
        self.pending_action.take()
    }

    /// Take (remove and return) a pending venv action, if any.
    pub fn take_pending_venv_action(&mut self) -> Option<VenvAction> {
        self.pending_venv_action.take()
    }

    /// Returns `true` while a background task is in-flight.
    pub fn is_busy(&self) -> bool {
        self.bg_rx.is_some()
    }

    /// Returns `true` while the UV version info is being fetched in the background.
    pub fn is_uv_info_loading(&self) -> bool {
        self.uv_info_rx.is_some()
    }

    /// Scroll the packages list in the detail panel down by one row.
    /// `total` is the number of packages in the currently selected venv.
    pub fn scroll_pkg_down(&mut self, total: usize) {
        if self.pkg_scroll + 1 < total {
            self.pkg_scroll += 1;
        }
    }

    /// Scroll the packages list in the detail panel up by one row.
    pub fn scroll_pkg_up(&mut self) {
        self.pkg_scroll = self.pkg_scroll.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app<'a>() -> App<'a> {
        App::new(vec![], true, Some("uv 0.5.0".to_string()))
    }

    #[test]
    fn test_tab_cycling() {
        let mut app = make_app();
        assert_eq!(app.tab, Tab::Environments);
        app.next_tab();
        assert_eq!(app.tab, Tab::UvInfo);
        app.next_tab();
        assert_eq!(app.tab, Tab::Environments);
    }

    #[test]
    fn test_prev_tab_cycling() {
        let mut app = make_app();
        app.prev_tab();
        assert_eq!(app.tab, Tab::UvInfo);
        app.prev_tab();
        assert_eq!(app.tab, Tab::Environments);
    }

    #[test]
    fn test_navigation_empty() {
        let mut app = make_app();
        app.next_item();
        assert_eq!(app.selected, 0);
        app.prev_item();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_pending_action_none_by_default() {
        let mut app = make_app();
        assert!(app.take_pending_action().is_none());
    }

    #[test]
    fn test_pending_action_take() {
        let mut app = make_app();
        app.pending_action = Some(UvAction::Install);
        assert_eq!(app.take_pending_action(), Some(UvAction::Install));
        // Should be cleared after taking.
        assert!(app.take_pending_action().is_none());
    }

    #[test]
    fn test_pending_action_variants() {
        let mut app = make_app();

        app.pending_action = Some(UvAction::Update);
        assert_eq!(app.take_pending_action(), Some(UvAction::Update));

        app.pending_action = Some(UvAction::Uninstall);
        assert_eq!(app.take_pending_action(), Some(UvAction::Uninstall));
    }

    #[test]
    fn test_pending_venv_action_none_by_default() {
        let mut app = make_app();
        assert!(app.take_pending_venv_action().is_none());
    }

    #[test]
    fn test_pending_venv_action_take() {
        let mut app = make_app();
        let action = VenvAction::Create {
            name: "myenv".to_string(),
            version: "3.12".to_string(),
            packages: vec![],
            default_pkgs: false,
            req_file: None,
        };
        app.pending_venv_action = Some(action.clone());
        assert_eq!(app.take_pending_venv_action(), Some(action));
        // Should be cleared after taking.
        assert!(app.take_pending_venv_action().is_none());
    }

    #[test]
    fn test_pending_venv_action_variants() {
        let mut app = make_app();

        app.pending_venv_action = Some(VenvAction::Delete);
        assert_eq!(app.take_pending_venv_action(), Some(VenvAction::Delete));

        app.pending_venv_action = Some(VenvAction::Activate);
        assert_eq!(app.take_pending_venv_action(), Some(VenvAction::Activate));
    }

    #[test]
    fn test_is_busy_default_false() {
        let app = make_app();
        assert!(!app.is_busy());
    }

    #[test]
    fn test_is_busy_true_when_rx_set() {
        let mut app = make_app();
        let (tx, rx) = tokio::sync::oneshot::channel::<Result<(), String>>();
        app.bg_rx = Some(rx);
        assert!(app.is_busy());
        drop(tx); // avoid leak warning
    }

    #[test]
    fn test_is_uv_info_loading_default_false() {
        let app = make_app();
        assert!(!app.is_uv_info_loading());
    }

    #[test]
    fn test_is_uv_info_loading_true_when_rx_set() {
        let mut app = make_app();
        let (_tx, rx) =
            tokio::sync::oneshot::channel::<(Option<String>, Option<String>)>();
        app.uv_info_rx = Some(rx);
        assert!(app.is_uv_info_loading());
    }

    #[test]
    fn test_confirm_dialog_none_by_default() {
        let app = make_app();
        assert!(app.confirm_dialog.is_none());
    }

    // ── Navigation with venvs ────────────────────────────────────────────────

    fn make_app_with_venvs<'a>() -> App<'a> {
        use pylot_shared::uvvenv::UvVenv;
        use std::borrow::Cow;
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
            UvVenv::new(
                Cow::Owned("env3".to_string()),
                "".to_string(),
                "3.10".to_string(),
                vec![],
                false,
            ),
        ];
        App::new(venvs, true, Some("uv 0.5.0".to_string()))
    }

    #[test]
    fn test_next_item_with_venvs() {
        let mut app = make_app_with_venvs();
        assert_eq!(app.selected, 0);
        app.next_item();
        assert_eq!(app.selected, 1);
        app.next_item();
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn test_next_item_wraps_around() {
        let mut app = make_app_with_venvs();
        app.selected = 2; // last item
        app.next_item();
        assert_eq!(app.selected, 0); // wraps to first
    }

    #[test]
    fn test_prev_item_with_venvs() {
        let mut app = make_app_with_venvs();
        app.selected = 2;
        app.prev_item();
        assert_eq!(app.selected, 1);
        app.prev_item();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_prev_item_wraps_around() {
        let mut app = make_app_with_venvs();
        assert_eq!(app.selected, 0);
        app.prev_item();
        assert_eq!(app.selected, 2); // wraps to last
    }

    #[test]
    fn test_navigation_does_nothing_on_uv_tab() {
        let mut app = make_app_with_venvs();
        app.next_tab(); // switch to UvInfo
        app.next_item();
        assert_eq!(app.selected, 0); // unchanged on UV tab
        app.prev_item();
        assert_eq!(app.selected, 0);
    }

    // ── App status_message field ─────────────────────────────────────────────

    #[test]
    fn test_status_message_none_by_default() {
        let app = make_app();
        assert!(app.status_message.is_none());
    }

    #[test]
    fn test_status_message_can_be_set() {
        let mut app = make_app();
        app.status_message = Some(("done".to_string(), false, Instant::now()));
        assert!(matches!(app.status_message, Some((ref msg, false, _)) if msg == "done"));
    }

    // ── App bg_task_name field ───────────────────────────────────────────────

    #[test]
    fn test_bg_task_name_none_by_default() {
        let app = make_app();
        assert!(app.bg_task_name.is_none());
    }

    // ── pkg_scroll ───────────────────────────────────────────────────────────

    #[test]
    fn test_pkg_scroll_defaults_to_zero() {
        let app = make_app();
        assert_eq!(app.pkg_scroll, 0);
    }

    #[test]
    fn test_pkg_scroll_up_at_zero_stays_zero() {
        let mut app = make_app();
        app.scroll_pkg_up();
        assert_eq!(app.pkg_scroll, 0);
    }

    #[test]
    fn test_pkg_scroll_down_increments() {
        let mut app = make_app();
        app.scroll_pkg_down(5);
        assert_eq!(app.pkg_scroll, 1);
        app.scroll_pkg_down(5);
        assert_eq!(app.pkg_scroll, 2);
    }

    #[test]
    fn test_pkg_scroll_down_capped_at_total_minus_one() {
        let mut app = make_app();
        // With total=3, max scroll is 2.
        app.scroll_pkg_down(3);
        app.scroll_pkg_down(3);
        app.scroll_pkg_down(3); // should stop at 2
        assert_eq!(app.pkg_scroll, 2);
    }

    #[test]
    fn test_pkg_scroll_up_decrements() {
        let mut app = make_app();
        app.pkg_scroll = 3;
        app.scroll_pkg_up();
        assert_eq!(app.pkg_scroll, 2);
    }

    #[test]
    fn test_next_item_resets_pkg_scroll() {
        let mut app = make_app_with_venvs();
        app.pkg_scroll = 5;
        app.next_item();
        assert_eq!(app.pkg_scroll, 0);
    }

    #[test]
    fn test_prev_item_resets_pkg_scroll() {
        let mut app = make_app_with_venvs();
        app.selected = 2;
        app.pkg_scroll = 5;
        app.prev_item();
        assert_eq!(app.pkg_scroll, 0);
    }

    // ── App pkg_dialog and pkg_search fields ─────────────────────────────────

    #[test]
    fn test_pkg_dialog_none_by_default() {
        let app = make_app();
        assert!(app.pkg_dialog.is_none());
    }

    #[test]
    fn test_pkg_search_none_by_default() {
        let app = make_app();
        assert!(app.pkg_search.is_none());
    }

    #[test]
    fn test_pkg_search_can_be_set() {
        let mut app = make_app();
        app.pkg_search = Some("requests".to_string());
        assert_eq!(app.pkg_search.as_deref(), Some("requests"));
    }
}
