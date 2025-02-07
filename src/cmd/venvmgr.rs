use super::utils;
use crate::cfg::settings;

pub struct Venv {
    name: String,
    python_version: String,
    packages: Vec<String>,
}

impl Venv {
    pub fn new(name: String, python_version: String, packages: Vec<String>) -> Self {
        Venv {
            name,
            python_version,
            packages,
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
        //let mut child = utils::create_child_cmd("uv", "venv", args);
        //utils::run_command(&mut child).await;
        if self.packages.len() > 0 {
            for package in &self.packages {
                println!("Installing package: {}", package);
            }
        }
        std::env::set_current_dir(pwd).unwrap();
    }

    pub async fn delete(&self) {
        println!("Deleting virtual environment: {}", self.name);
    }

    pub async fn list() {
        println!("Listing virtual environments");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_venv() {
        let venv = Venv::new("test_venv".to_string(), "3.8".to_string(), vec![]);
        assert_eq!(venv.name, "test_venv");
        assert_eq!(venv.python_version, "3.8");
    }

    #[tokio::test]
    async fn test_venv_clean() {
        let venv = Venv::new("test_venv_clean".to_string(), "3.9".to_string(), vec!["numpy".to_string(), "pandas".to_string()]);
        assert_eq!(venv.name, "test_venv_clean");
        assert_eq!(venv.python_version, "3.9");
        assert_eq![venv.packages ,&["numpy", "pandas"]]
    }
}
