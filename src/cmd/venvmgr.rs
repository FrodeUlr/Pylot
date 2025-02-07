use super::utils;
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
        let pwd = std::env::current_dir().unwrap();
        // set pwd to settings venvs_path
        let path = shellexpand::tilde(&settings.venvs_path).to_string();
        std::env::set_current_dir(&path).unwrap();
        let args = &[
            self.name.as_str(),
            "--python",
            self.python_version.as_str(),
        ];
        println!("Creating virtual environment: {}", self.name);
        let mut child = utils::create_child_cmd("uv", "venv", args);
        utils::run_command(&mut child).await;
        std::env::set_current_dir(pwd).unwrap();
    }

    pub async fn delete(&self) {
        println!("Deleting virtual environment: {}", self.name);
    }

    pub async fn list() {
        println!("Listing virtual environments");
    }
}
