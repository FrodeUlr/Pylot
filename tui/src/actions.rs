/// UV management actions that can be triggered from the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UvAction {
    Install,
    Update,
    Uninstall,
}

/// Which destructive action is awaiting user confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Delete the named virtual environment.
    DeleteVenv(String),
    /// Uninstall Astral UV.
    UninstallUv,
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
        /// Optional path to a requirements.txt file to install after creation.
        req_file: Option<String>,
    },
    Delete,
    Activate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_action_debug() {
        let action = UvAction::Install;
        assert_eq!(format!("{:?}", action), "Install");
    }

    #[test]
    fn test_confirm_action_debug() {
        let action = ConfirmAction::DeleteVenv("myenv".to_string());
        assert_eq!(format!("{:?}", action), "DeleteVenv(\"myenv\")");
    }

    #[test]
    fn test_venv_action_debug() {
        let action = VenvAction::Create {
            name: "env".to_string(),
            version: "3.10".to_string(),
            packages: vec!["numpy".to_string()],
            default_pkgs: true,
            req_file: Some("requirements.txt".to_string()),
        };
        assert!(format!("{:?}", action).contains("Create"));
    }
}
