use config::{Config, File};

#[derive(Debug, serde::Deserialize)]
struct Settings {
    location: String,
}

pub async fn init() {
    let settings = Config::builder()
        .add_source(File::with_name("settings"))
        .build()
        .expect("Failed to build settings");

    let setting = settings.try_deserialize::<Settings>().expect("Failed to deserialize settings");

    println!("Settings: {:?}", setting);
    println!("Location: {}", setting.location);
}
