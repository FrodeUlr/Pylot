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
            "venv",
            self.name.as_str(),
            "--python",
            self.python_version.as_str(),
        ];
        println!("Creating virtual environment: {}", self.name);
        let mut child = utils::create_child_cmd("uv", args);
        utils::run_command(&mut child).await;
        if self.packages.len() > 0 {
            let venn_path = shellexpand::tilde(&settings.venvs_path).to_string();
            let mut child2 = if cfg!(target_os = "windows") {
                // create path from venv name and script location
                let path = format!("{}/{}/scripts/activate", venn_path, self.name);
                println!("Activating virtual environment: {}", path);
                utils::create_child_cmd("&", &[path.as_str()])
            } else {
                let path = format!("{}/{}/bin/activate",venn_path, self.name);
                println!("Activating virtual environment: {}", path);
                utils::create_child_cmd("source", &[path.as_str()])
            };
            utils::run_command(&mut child2).await;
            for package in &self.packages {
                println!("Installing package: {}", package);
                let mut child3 = utils::create_child_cmd("uv", &["pip", "install", package]);
                utils::run_command(&mut child3).await;
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
