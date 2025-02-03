use config::{Config, File};
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    pub location: String,
}

static SETTINGS: Lazy<Mutex<Settings>> = Lazy::new(|| Mutex::new(Settings {
    location: String::from(""),
}));

impl Settings {
    pub async fn init() {
        let settings = Config::builder()
            .add_source(File::with_name("settings"))
            .build()
            .expect("Failed to build settings");

        let new_settings: Settings = settings.try_deserialize().expect("Failed to deserialize settings");

        let mut settings_lock = SETTINGS.lock().expect("Failed to lock settings");
        *settings_lock = new_settings;
    }

        pub fn get_settings() -> Settings {
        let settings_lock = SETTINGS.lock().expect("Failed to lock settings");
        settings_lock.clone()
    }
}
