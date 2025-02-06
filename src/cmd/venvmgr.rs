use crate::cfg::settings;

pub struct Venv {
    name: String,
    python_version: String,
    clean: bool,
}

impl Venv {
    pub fn new(name: String, python_version: String, clean: bool) -> Self {
        Venv {
            name,
            python_version,
            clean,
        }
    }

    pub async fn create(&self) {
        let settings = settings::Settings::get_settings();
        println!("Creating virtual environment: {}", self.name);
        println!("Python version: {}", self.python_version);
        println!("Clean: {}", self.clean);
        println!("Settings: {:?}", settings);
        println!("Settings location: {:?}", settings.venvs_path);
    }

    pub async fn delete(&self) {
        println!("Deleting virtual environment: {}", self.name);
    }

    pub async fn list() {
        println!("Listing virtual environments");
    }
}
