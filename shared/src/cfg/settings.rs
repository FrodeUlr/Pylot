use config::{Config, File, FileFormat};
use std::{
    env,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

/// Application settings deserialized from `settings.toml`.
///
/// The settings file is expected to live in the same directory as the compiled
/// executable.  If the file does not exist it is created automatically with
/// default values on first run.
///
/// # Example `settings.toml`
///
/// ```toml
/// venvs_path = "~/pylot/venvs"
/// default_pkgs = ["numpy", "requests"]
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    /// The directory where managed virtual environments are stored.
    ///
    /// Tilde expansion is applied, so `~/pylot/venvs` works on all platforms.
    #[serde(default = "default_venv_path")]
    pub venvs_path: String,
    /// Packages that are installed automatically when a virtual environment is
    /// created with the `--default` flag.
    #[serde(default)]
    pub default_pkgs: Vec<String>,
}

fn default_venv_path() -> String {
    String::from("~/pylot/venvs")
}

static SETTINGS: LazyLock<Mutex<Settings>> = LazyLock::new(|| Mutex::new(Settings::default()));

impl Default for Settings {
    fn default() -> Self {
        Settings {
            venvs_path: default_venv_path(),
            default_pkgs: Vec::new(),
        }
    }
}

impl Settings {
    /// Load settings from `settings.toml` next to the running executable and
    /// store them in the process-wide singleton.
    ///
    /// This must be called once at application start-up (before any call to
    /// [`Settings::get_settings`]).  If the file does not exist it is created
    /// with default values.
    pub async fn init() {
        let exe_dir = Self::get_exe_dir(env::current_exe);
        Self::init_with_dir(&exe_dir).await;
    }

    async fn init_with_dir(dir: &Path) {
        let settings_path = dir.join("settings.toml");

        if !settings_path.exists() {
            match toml::to_string_pretty(&Settings::default())
                .map_err(|e| format!("Failed to serialize default settings: {}", e))
                .and_then(|s| {
                    std::fs::write(&settings_path, s)
                        .map_err(|e| format!("Failed to write default settings.toml: {}", e))
                }) {
                Ok(()) => {}
                Err(e) => eprintln!("{}", e),
            }
        }

        let settings = Config::builder()
            .add_source(File::from(settings_path).format(FileFormat::Toml))
            .build()
            .unwrap_or_else(|_| {
                eprintln!("Settings.toml is invalid, using defaults");
                Config::default()
            });

        let new_settings: Settings = settings
            .try_deserialize()
            .unwrap_or_else(|_| Settings::default());

        new_settings.validate_venv_path();

        if let Ok(mut settings_lock) = SETTINGS.lock() {
            *settings_lock = new_settings;
        }
    }

    /// Return a clone of the current process-wide settings.
    ///
    /// Falls back to [`Settings::default`] if the internal mutex is poisoned.
    pub fn get_settings() -> Settings {
        SETTINGS
            .lock()
            .map(|s| s.clone())
            .unwrap_or_else(|_| Settings::default())
    }

    /// Ensure that [`Settings::venvs_path`] exists on disk, creating the
    /// directory (and all missing parents) if necessary.
    pub fn validate_venv_path(&self) {
        let mut path = self.venvs_path.clone();
        if path.starts_with("~") {
            path = shellexpand::tilde(&path).to_string();
        }
        if !Path::new(&path).exists() {
            eprintln!("Creating venvs folder: {}", path);
            if let Err(e) = std::fs::create_dir_all(&path) {
                eprintln!("Failed to create venvs folder: {}", e);
            }
        }
    }

    /// Return the directory that contains the running executable.
    ///
    /// Accepts a callable `current_exe_fn` so the logic can be tested without
    /// touching the real filesystem.
    pub fn get_exe_dir<F>(current_exe_fn: F) -> PathBuf
    where
        F: Fn() -> std::io::Result<PathBuf>,
    {
        match current_exe_fn() {
            Ok(exe_path) => exe_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from(".")),
            Err(_) => {
                eprintln!("Could not determine the executable directory");
                PathBuf::from(".")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        sync::Mutex,
    };

    use crate::logger;

    use super::*;

    #[test]
    fn test_default_venv_path() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let settings = Settings::default();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
    }

    #[test]
    fn test_validate_venv_path() {
        logger::initialize_logger(log::LevelFilter::Trace);
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
        logger::initialize_logger(log::LevelFilter::Trace);
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
        logger::initialize_logger(log::LevelFilter::Trace);
        let settings = Settings::default();
        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(settings.default_pkgs, empty_vec);
    }

    #[tokio::test]
    async fn test_init() {
        logger::initialize_logger(log::LevelFilter::Trace);
        Settings::init().await;
        let settings = Settings::get_settings();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
    }

    #[test]
    fn test_settings_deserialize() {
        logger::initialize_logger(log::LevelFilter::Trace);
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
        logger::initialize_logger(log::LevelFilter::Trace);
        let toml_str = r#"
            default_pkgs = ["requests"]
        "#;

        let settings: Settings = toml::from_str(toml_str).unwrap();
        assert_eq!(settings.venvs_path, "~/pylot/venvs");
        assert_eq!(settings.default_pkgs, vec!["requests"]);
    }

    #[test]
    fn test_settings_deserialize_invalid() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let toml_str = r#"
            venvs_path = 123
            default_pkgs = "not_a_list"
        "#;

        let result: Result<Settings, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_exe_dir() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let fake_current_exe = || Ok(PathBuf::from("/usr/local/bin/pylot"));
        let exe_dir = Settings::get_exe_dir(fake_current_exe);
        assert_eq!(exe_dir, PathBuf::from("/usr/local/bin"));
    }

    #[test]
    fn test_get_exe_dir_error() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let fake_current_exe = || Err(std::io::Error::other("error"));
        let exe_dir = Settings::get_exe_dir(fake_current_exe);
        assert_eq!(exe_dir, PathBuf::from("."));
    }

    #[tokio::test]
    async fn test_init_creates_default_settings_toml_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let settings_path = tmp.path().join("settings.toml");

        assert!(!settings_path.exists(), "settings.toml should not exist yet");

        Settings::init_with_dir(tmp.path()).await;

        assert!(settings_path.exists(), "settings.toml should have been created");

        let content = std::fs::read_to_string(&settings_path).unwrap();
        assert!(
            content.contains("venvs_path"),
            "generated settings.toml should contain venvs_path key"
        );
    }
}
