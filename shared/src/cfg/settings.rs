use config::{Config, File, FileFormat};
use once_cell::sync::Lazy;
use std::{
    env,
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    #[serde(default = "default_venv_path")]
    pub venvs_path: String,
    #[serde(default)]
    pub default_pkgs: Vec<String>,
}

fn default_venv_path() -> String {
    String::from("~/pylot/venvs")
}

static SETTINGS: Lazy<Mutex<Settings>> = Lazy::new(|| Mutex::new(Settings::default()));

impl Default for Settings {
    fn default() -> Self {
        Settings {
            venvs_path: default_venv_path(),
            default_pkgs: Vec::new(),
        }
    }
}

impl Settings {
    pub async fn init() {
        let exe_dir = Self::get_exe_dir(env::current_exe);

        let settings_path = exe_dir.join("settings.toml");

        let settings = Config::builder()
            .add_source(File::from(settings_path).format(FileFormat::Toml))
            .build()
            .unwrap_or_else(|_| {
                println!("Settings.toml missing or invalid");
                Config::default()
            });

        let new_settings: Settings = settings
            .try_deserialize()
            .unwrap_or_else(|_| Settings::default());

        new_settings.validate_venv_path();

        let mut settings_lock = SETTINGS.lock().expect("Failed to lock settings");
        *settings_lock = new_settings;
    }

    pub fn get_settings() -> Settings {
        let settings_lock = SETTINGS.lock().expect("Failed to lock settings");
        settings_lock.clone()
    }

    fn validate_venv_path(&self) {
        let mut path = self.venvs_path.clone();
        if path.starts_with("~") {
            path = shellexpand::tilde(&path).to_string();
        }
        if !Path::new(&path).exists() {
            println!("Creating venvs folder: {}", path);
            std::fs::create_dir_all(&path).expect("Failed to create venvs folder");
        }
    }

    fn get_exe_dir<F>(current_exe_fn: F) -> PathBuf
    where
        F: Fn() -> std::io::Result<PathBuf>,
    {
        let exe_dir = match current_exe_fn() {
            Ok(exe_path) => exe_path.parent().unwrap().to_path_buf(),
            Err(_) => {
                println!("Could not determine the executable directory");
                PathBuf::from(".")
            }
        };
        exe_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_venv_path() {
        let settings = Settings::default();
        assert_eq!(settings.venvs_path, "~/pymngr/venvs");
    }

    #[test]
    fn test_validate_venv_path() {
        let settings = Settings {
            venvs_path: "~/pymngr/venvs".to_string(),
            default_pkgs: vec![],
        };
        settings.validate_venv_path();
        let expected_path = shellexpand::tilde("~/pymngr/venvs").to_string();
        assert!(Path::new(&expected_path).exists());
    }

    #[test]
    fn test_get_settings() {
        let settings = Settings {
            venvs_path: "~/pymngr/venvs".to_string(),
            default_pkgs: vec![],
        };
        let settings_lock = Mutex::new(settings);
        let settings = settings_lock.lock().unwrap();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
    }

    #[test]
    fn test_default_pkgs() {
        let settings = Settings::default();
        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(settings.default_pkgs, empty_vec);
    }

    #[tokio::test]
    async fn test_init() {
        Settings::init().await;
        let settings = Settings::get_settings();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
    }

    #[test]
    fn test_settings_deserialize() {
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
        let toml_str = r#"
            default_pkgs = ["requests"]
        "#;

        let settings: Settings = toml::from_str(toml_str).unwrap();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
        assert_eq!(settings.default_pkgs, vec!["requests"]);
    }

    #[test]
    fn test_settings_deserialize_invalid() {
        let toml_str = r#"
            venvs_path = 123
            default_pkgs = "not_a_list"
        "#;

        let result: Result<Settings, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_exe_dir() {
        let fake_current_exe = || Ok(PathBuf::from("/usr/local/bin/pymngr"));
        let exe_dir = Settings::get_exe_dir(fake_current_exe);
        assert_eq!(exe_dir, PathBuf::from("/usr/local/bin"));
    }

    #[test]
    fn test_get_exe_dir_error() {
        let fake_current_exe = || Err(std::io::Error::other("error"));
        let exe_dir = Settings::get_exe_dir(fake_current_exe);
        assert_eq!(exe_dir, PathBuf::from("."));
    }
}
