mod helpers;

#[cfg(test)]
mod tests {
    use shared::{
        constants::{PWSH_CMD, SH_CMD},
        uvvenv::UvVenv,
    };

    use crate::helpers::setup_logger;

    #[tokio::test]
    async fn test_venv() {
        setup_logger();
        let venv = UvVenv::new(
            "test_venv".to_string(),
            "".to_string(),
            "3.8".to_string(),
            vec![],
            false,
        );
        assert_eq!(venv.name, "test_venv");
        assert_eq!(venv.python_version, "3.8");
    }

    #[tokio::test]
    async fn test_venv_clean() {
        setup_logger();
        let venv = UvVenv::new(
            "test_venv_clean".to_string(),
            "".to_string(),
            "3.9".to_string(),
            vec!["numpy".to_string(), "pandas".to_string()],
            false,
        );
        assert_eq!(venv.name, "test_venv_clean");
        assert_eq!(venv.python_version, "3.9");
        assert_eq![venv.packages, &["numpy", "pandas"]]
    }

    #[tokio::test]
    async fn test_generate_command() {
        setup_logger();
        let venv = UvVenv::new(
            "test_venv_cmd".to_string(),
            "".to_string(),
            "3.10".to_string(),
            vec!["requests".to_string()],
            true,
        );
        let (cmd, run, agr_str) = venv
            .generate_command(
                vec!["requests".to_string(), "flask".to_string()],
                "/home/user/.virtualenvs".to_string(),
            )
            .await;
        if cfg!(target_os = "windows") {
            assert_eq!(cmd, PWSH_CMD);
            assert_eq!(run, "-Command");
            assert!(agr_str.contains("activate.ps1"));
            assert!(agr_str.contains("uv pip install requests flask"));
        } else {
            assert_eq!(cmd, SH_CMD);
            assert_eq!(run, "-c");
            assert!(agr_str.contains("activate"));
            assert!(agr_str.contains("uv pip install requests flask"));
        }
    }

    #[test]
    fn test_get_settings_pwd_args() {
        setup_logger();
        let pwd_start = std::env::current_dir().unwrap();
        let venv = UvVenv::new(
            "test_venv_args".to_string(),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        if let Some((pwd, args)) = venv.get_pwd_args() {
            assert_eq!(args[0], "venv");
            assert_eq!(args[1], "test_venv_args");
            assert_eq!(args[2], "--python");
            assert_eq!(args[3], "3.11");
            assert_eq!(pwd, pwd_start);
        } else {
            panic!("get_pwd_args returned None");
        }
    }
}
