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

    pub fn validate_venv_path(&self) {
        let mut path = self.venvs_path.clone();
        if path.starts_with("~") {
            path = shellexpand::tilde(&path).to_string();
        }
        if !Path::new(&path).exists() {
            println!("Creating venvs folder: {}", path);
            std::fs::create_dir_all(&path).expect("Failed to create venvs folder");
        }
    }

    pub fn get_exe_dir<F>(current_exe_fn: F) -> PathBuf
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
