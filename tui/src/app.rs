use shared::virtualenv::uvvenv::UvVenv;

/// UV management actions that can be triggered from the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UvAction {
    Install,
    Update,
    Uninstall,
}

/// Which field of the create-venv dialog is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateField {
    Name,
    Version,
    Packages,
    DefaultPkgs,
}

impl CreateField {
    /// Advance to the next field (wraps around).
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Version,
            Self::Version => Self::Packages,
            Self::Packages => Self::DefaultPkgs,
            Self::DefaultPkgs => Self::Name,
        }
    }

    /// Go to the previous field (wraps around).
    pub fn prev(self) -> Self {
        match self {
            Self::Name => Self::DefaultPkgs,
            Self::Version => Self::Name,
            Self::Packages => Self::Version,
            Self::DefaultPkgs => Self::Packages,
        }
    }
}

/// In-TUI form state for creating a new virtual environment
pub struct CreateDialog {
    /// Currently focused input field
    pub field: CreateField,
    pub name: String,
    pub version: String,
    /// Raw comma-separated packages string as the user types it
    pub packages: String,
    pub default_pkgs: bool,
}

impl CreateDialog {
    pub fn new(default_version: &str) -> Self {
        CreateDialog {
            field: CreateField::Name,
            name: String::new(),
            version: default_version.to_string(),
            packages: String::new(),
            default_pkgs: false,
        }
    }

    /// Push a character into the currently focused text field (no-op for bool field).
    pub fn push_char(&mut self, c: char) {
        match self.field {
            CreateField::Name => self.name.push(c),
            CreateField::Version => self.version.push(c),
            CreateField::Packages => self.packages.push(c),
            CreateField::DefaultPkgs => {}
        }
    }

    /// Delete the last character of the currently focused text field.
    pub fn pop_char(&mut self) {
        match self.field {
            CreateField::Name => { self.name.pop(); }
            CreateField::Version => { self.version.pop(); }
            CreateField::Packages => { self.packages.pop(); }
            CreateField::DefaultPkgs => {}
        }
    }

    /// Toggle the boolean default-packages field.
    pub fn toggle_default(&mut self) {
        self.default_pkgs = !self.default_pkgs;
    }

    /// Collect packages from the raw comma-separated string.
    pub fn parsed_packages(&self) -> Vec<String> {
        self.packages
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Return the effective Python version: the user's input, or `DEFAULT_PYTHON_VERSION`
    /// if the version field was left blank.
    pub fn effective_version(&self) -> String {
        let v = self.version.trim();
        if v.is_empty() {
            shared::constants::DEFAULT_PYTHON_VERSION.to_string()
        } else {
            v.to_string()
        }
    }
}

/// Venv management actions that can be triggered from the TUI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VenvAction {
    /// Create a new venv with the given parameters (collected from the in-TUI dialog).
    Create {
        name: String,
        version: String,
        packages: Vec<String>,
        default_pkgs: bool,
    },
    Delete,
    Activate,
}

/// Tab identifiers for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Environments,
    UvInfo,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[Tab::Environments, Tab::UvInfo];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Environments => "Environments",
            Tab::UvInfo => "UV Info",
        }
    }
}

/// Application state for the TUI
pub struct App<'a> {
    pub tab: Tab,
    pub venvs: Vec<UvVenv<'a>>,
    pub selected: usize,
    pub uv_installed: bool,
    pub uv_version: Option<String>,
    pub pending_action: Option<UvAction>,
    pub pending_venv_action: Option<VenvAction>,
    /// When `Some`, the create-venv dialog is open.
    pub create_dialog: Option<CreateDialog>,
}

impl<'a> App<'a> {
    pub fn new(
        venvs: Vec<UvVenv<'a>>,
        uv_installed: bool,
        uv_version: Option<String>,
    ) -> Self {
        App {
            tab: Tab::Environments,
            venvs,
            selected: 0,
            uv_installed,
            uv_version,
            pending_action: None,
            pending_venv_action: None,
            create_dialog: None,
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
        }
    }

    pub fn prev_item(&mut self) {
        if self.tab == Tab::Environments && !self.venvs.is_empty() {
            if self.selected == 0 {
                self.selected = self.venvs.len() - 1;
            } else {
                self.selected -= 1;
            }
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
    fn test_tab_titles() {
        assert_eq!(Tab::Environments.title(), "Environments");
        assert_eq!(Tab::UvInfo.title(), "UV Info");
    }

    #[test]
    fn test_all_tabs() {
        assert_eq!(Tab::ALL.len(), 2);
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
    fn test_create_dialog_new() {
        let d = CreateDialog::new("3.12");
        assert_eq!(d.field, CreateField::Name);
        assert_eq!(d.version, "3.12");
        assert!(d.name.is_empty());
        assert!(!d.default_pkgs);
    }

    #[test]
    fn test_create_dialog_push_pop() {
        let mut d = CreateDialog::new("3.12");
        d.push_char('m');
        d.push_char('y');
        assert_eq!(d.name, "my");
        d.pop_char();
        assert_eq!(d.name, "m");
    }

    #[test]
    fn test_create_dialog_field_cycling() {
        assert_eq!(CreateField::Name.next(), CreateField::Version);
        assert_eq!(CreateField::Version.next(), CreateField::Packages);
        assert_eq!(CreateField::Packages.next(), CreateField::DefaultPkgs);
        assert_eq!(CreateField::DefaultPkgs.next(), CreateField::Name);

        assert_eq!(CreateField::Name.prev(), CreateField::DefaultPkgs);
        assert_eq!(CreateField::DefaultPkgs.prev(), CreateField::Packages);
    }

    #[test]
    fn test_create_dialog_toggle_default() {
        let mut d = CreateDialog::new("3.12");
        assert!(!d.default_pkgs);
        d.toggle_default();
        assert!(d.default_pkgs);
        d.toggle_default();
        assert!(!d.default_pkgs);
    }

    #[test]
    fn test_create_dialog_parsed_packages() {
        let mut d = CreateDialog::new("3.12");
        d.packages = "requests, flask , ".to_string();
        assert_eq!(d.parsed_packages(), vec!["requests", "flask"]);
    }

    #[test]
    fn test_create_dialog_effective_version() {
        let d = CreateDialog::new("3.12");
        assert_eq!(d.effective_version(), "3.12");

        let mut d2 = CreateDialog::new("3.12");
        d2.version = "  ".to_string();
        // Blank version falls back to DEFAULT_PYTHON_VERSION.
        assert_eq!(d2.effective_version(), shared::constants::DEFAULT_PYTHON_VERSION);
    }
}
