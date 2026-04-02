use crate::actions::ConfirmAction;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Whether the package management dialog is adding or removing packages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkgDialogMode {
    Add,
    Remove,
}

/// In-TUI form for adding or removing packages from the selected virtual environment
pub struct PkgDialog {
    pub mode: PkgDialogMode,
    /// Raw comma-separated packages string as the user types it
    pub input: String,
}

impl PkgDialog {
    pub fn new(mode: PkgDialogMode) -> Self {
        PkgDialog {
            mode,
            input: String::new(),
        }
    }

    /// Push a character into the input field.
    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    /// Delete the last character from the input field.
    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    /// Collect packages from the raw comma-separated string.
    pub fn parsed_packages(&self) -> Vec<String> {
        self.input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Returns the dialog title.
    pub fn title(&self) -> &'static str {
        match self.mode {
            PkgDialogMode::Add => " Add Packages ",
            PkgDialogMode::Remove => " Remove Packages ",
        }
    }
}

/// Simple yes/no confirmation overlay
pub struct ConfirmDialog {
    pub action: ConfirmAction,
}

impl ConfirmDialog {
    pub fn new(action: ConfirmAction) -> Self {
        ConfirmDialog { action }
    }

    /// Returns the human-readable question shown in the dialog.
    pub fn message(&self) -> String {
        match &self.action {
            ConfirmAction::DeleteVenv(name) => {
                format!("Delete virtual environment '{}'?", name)
            }
            ConfirmAction::UninstallUv => "Uninstall Astral UV?".to_string(),
        }
    }
}

// Specifies which help menu is currently active (environments or UV info)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpMode {
    EnvHelp,
    UvHelp,
}

// Help dialog
pub struct HelpDialog {
    pub help_menu: HelpMode,
    pub height: u16,
    pub width: u16,
}

impl HelpDialog {
    // New dialog with dimensions based on the help menu type
    pub fn new(help_menu: HelpMode) -> Self {
        match help_menu {
            HelpMode::EnvHelp => HelpDialog {
                help_menu,
                height: 21,
                width: 60,
            },
            HelpMode::UvHelp => HelpDialog {
                help_menu,
                height: 17,
                width: 50,
            },
        }
    }
    // Create the help lines based on the active help menu, including global and footer lines
    pub fn lines(&self) -> Vec<Line<'static>> {
        let mut lines = self.global_help_lines();
        match self.help_menu {
            HelpMode::EnvHelp => lines.extend(self.env_help_lines()),
            HelpMode::UvHelp => lines.extend(self.uv_help_lines()),
        }
        lines.extend(self.footer_help_lines());
        lines
    }

    fn default_bulet_span(&self) -> Span<'static> {
        Span::raw("    - ")
    }

    fn global_help_lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Global:",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("Tab / ←→", Style::default().fg(Color::Yellow)),
                Span::raw(": Next/Previous tab"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("↑↓      ", Style::default().fg(Color::Yellow)),
                Span::raw(": Navigate"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("Esc / q ", Style::default().fg(Color::Yellow)),
                Span::raw(": Quit (or dismiss dialogs)"),
            ]),
        ]
    }

    fn footer_help_lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Esc / q", Style::default().fg(Color::Yellow)),
                Span::raw(": close"),
            ]),
        ]
    }

    fn env_help_lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Environments:",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(": Select environment"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("n    ", Style::default().fg(Color::Yellow)),
                Span::raw(": Create new environment"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("a    ", Style::default().fg(Color::Yellow)),
                Span::raw(": Add packages to selected environment"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("r    ", Style::default().fg(Color::Yellow)),
                Span::raw(": Remove packages from selected environment"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("d    ", Style::default().fg(Color::Yellow)),
                Span::raw(": Delete selected environment"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("/    ", Style::default().fg(Color::Yellow)),
                Span::raw(": Search for package in selected environment"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("j/k  ", Style::default().fg(Color::Yellow)),
                Span::raw(": Scroll package list up/down"),
            ]),
        ]
    }

    fn uv_help_lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "UV Info:",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("i", Style::default().fg(Color::Yellow)),
                Span::raw(": Install Astral UV"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw(": Uninstall Astral UV"),
            ]),
            Line::from(vec![
                self.default_bulet_span(),
                Span::styled("u", Style::default().fg(Color::Yellow)),
                Span::raw(": Update Astral UV to latest version"),
            ]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PkgDialog ────────────────────────────────────────────────────────────

    #[test]
    fn test_pkg_dialog_new_add() {
        let d = PkgDialog::new(PkgDialogMode::Add);
        assert_eq!(d.mode, PkgDialogMode::Add);
        assert!(d.input.is_empty());
    }

    #[test]
    fn test_pkg_dialog_new_remove() {
        let d = PkgDialog::new(PkgDialogMode::Remove);
        assert_eq!(d.mode, PkgDialogMode::Remove);
        assert!(d.input.is_empty());
    }

    #[test]
    fn test_pkg_dialog_push_pop() {
        let mut d = PkgDialog::new(PkgDialogMode::Add);
        d.push_char('r');
        d.push_char('e');
        assert_eq!(d.input, "re");
        d.pop_char();
        assert_eq!(d.input, "r");
    }

    #[test]
    fn test_pkg_dialog_pop_empty_no_panic() {
        let mut d = PkgDialog::new(PkgDialogMode::Remove);
        d.pop_char(); // must not panic
        assert!(d.input.is_empty());
    }

    #[test]
    fn test_pkg_dialog_parsed_packages() {
        let mut d = PkgDialog::new(PkgDialogMode::Add);
        d.input = "requests, flask , ".to_string();
        assert_eq!(d.parsed_packages(), vec!["requests", "flask"]);
    }

    #[test]
    fn test_pkg_dialog_parsed_packages_empty() {
        let d = PkgDialog::new(PkgDialogMode::Add);
        assert!(d.parsed_packages().is_empty());
    }

    #[test]
    fn test_pkg_dialog_title_add() {
        let d = PkgDialog::new(PkgDialogMode::Add);
        assert!(d.title().contains("Add"));
    }

    #[test]
    fn test_pkg_dialog_title_remove() {
        let d = PkgDialog::new(PkgDialogMode::Remove);
        assert!(d.title().contains("Remove"));
    }

    // ── ConfirmDialog ────────────────────────────────────────────────────────────

    #[test]
    fn test_confirm_dialog_message_delete_venv() {
        let d = ConfirmDialog::new(ConfirmAction::DeleteVenv("myenv".to_string()));
        assert!(d.message().contains("myenv"));
        assert!(d.message().contains("Delete"));
    }

    #[test]
    fn test_confirm_dialog_message_uninstall_uv() {
        let d = ConfirmDialog::new(ConfirmAction::UninstallUv);
        assert!(d.message().contains("Uninstall"));
    }

    // -─ HelpDialog ────────────────────────────────────────────────────────────

    #[test]
    fn test_help_dialog_new_env_help() {
        let d = HelpDialog::new(HelpMode::EnvHelp);
        assert_eq!(d.help_menu, HelpMode::EnvHelp);
        assert_eq!(d.height, 21);
        assert_eq!(d.width, 60);
    }

    #[test]
    fn test_help_dialog_new_uv_help() {
        let d = HelpDialog::new(HelpMode::UvHelp);
        assert_eq!(d.help_menu, HelpMode::UvHelp);
        assert_eq!(d.height, 17);
        assert_eq!(d.width, 50);
    }

    #[test]
    fn test_help_dialog_default_bullet_span() {
        let d = HelpDialog::new(HelpMode::EnvHelp);
        let span = d.default_bulet_span();
        assert_eq!(span.content, "    - ");
    }

    #[test]
    fn test_help_dialog_env_help_lines() {
        let d = HelpDialog::new(HelpMode::EnvHelp);
        let lines = d.env_help_lines();
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_help_dialog_uv_help_lines() {
        let d = HelpDialog::new(HelpMode::UvHelp);
        let lines = d.uv_help_lines();
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_help_dialog_global_help_lines() {
        let d = HelpDialog::new(HelpMode::EnvHelp);
        let lines = d.global_help_lines();
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_help_dialog_footer_help_lines() {
        let d = HelpDialog::new(HelpMode::UvHelp);
        let lines = d.footer_help_lines();
        assert!(!lines.is_empty());
    }
}
