use shared::virtualenv::uvvenv::UvVenv;

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
}
