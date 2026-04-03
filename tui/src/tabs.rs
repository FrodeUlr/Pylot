use crate::dialogs::HelpMode;

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

    pub fn help_mode(self) -> HelpMode {
        match self {
            Tab::Environments => HelpMode::EnvHelp,
            Tab::UvInfo => HelpMode::UvHelp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_help_modes() {
        assert_eq!(Tab::Environments.help_mode(), HelpMode::EnvHelp);
        assert_eq!(Tab::UvInfo.help_mode(), HelpMode::UvHelp);
    }
}
