use crate::actions::ConfirmAction;

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
}
