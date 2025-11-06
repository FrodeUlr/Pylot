mod helpers;

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        sync::Mutex,
    };

    use shared::settings::Settings;

    use crate::helpers::setup_logger;

    #[test]
    fn test_default_venv_path() {
        setup_logger();
        let settings = Settings::default();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
    }

    #[test]
    fn test_validate_venv_path() {
        setup_logger();
        let settings = Settings {
            venvs_path: "~/pylot/venvs".to_string(),
            default_pkgs: vec![],
        };
        settings.validate_venv_path();
        let expected_path = shellexpand::tilde("~/pylot/venvs").to_string();
        assert!(Path::new(&expected_path).exists());
    }

    #[test]
    fn test_get_settings() {
        setup_logger();
        let settings = Settings {
            venvs_path: "~/pylot/venvs".to_string(),
            default_pkgs: vec![],
        };
        let settings_lock = Mutex::new(settings);
        let settings = settings_lock.lock().unwrap();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
    }

    #[test]
    fn test_default_pkgs() {
        setup_logger();
        let settings = Settings::default();
        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(settings.default_pkgs, empty_vec);
    }

    #[tokio::test]
    async fn test_init() {
        setup_logger();
        Settings::init().await;
        let settings = Settings::get_settings();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
    }

    #[test]
    fn test_settings_deserialize() {
        setup_logger();
        let toml_str = r#"
            venvs_path = "~/custom/venvs"
            default_pkgs = ["numpy", "pandas"]
        "#;

        let settings: Settings = toml::from_str(toml_str).unwrap();
        assert_eq!(settings.venvs_path, "~/custom/venvs");
        assert_eq!(settings.default_pkgs, vec!["numpy", "pandas"]);
    }

    #[test]
    fn test_settings_deserialize_missing_fields() {
        setup_logger();
        let toml_str = r#"
            default_pkgs = ["requests"]
        "#;

        let settings: Settings = toml::from_str(toml_str).unwrap();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
        assert_eq!(settings.default_pkgs, vec!["requests"]);
    }

    #[test]
    fn test_settings_deserialize_invalid() {
        setup_logger();
        let toml_str = r#"
            venvs_path = 123
            default_pkgs = "not_a_list"
        "#;

        let result: Result<Settings, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_exe_dir() {
        setup_logger();
        let fake_current_exe = || Ok(PathBuf::from("/usr/local/bin/pylot"));
        let exe_dir = Settings::get_exe_dir(fake_current_exe);
        assert_eq!(exe_dir, PathBuf::from("/usr/local/bin"));
    }

    #[test]
    fn test_get_exe_dir_error() {
        setup_logger();
        let fake_current_exe = || Err(std::io::Error::other("error"));
        let exe_dir = Settings::get_exe_dir(fake_current_exe);
        assert_eq!(exe_dir, PathBuf::from("."));
    }
}
