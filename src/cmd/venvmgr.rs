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
        let pkgs = self.packages.clone();
        if self.packages.len() > 0 {
            let venn_path = shellexpand::tilde(&settings.venvs_path).to_string();

            let vpath = if cfg!(target_os = "windows") {
                format!("{}/{}/scripts/activate", venn_path, self.name)
            } else {
                format!("{}/{}/bin/activate", venn_path, self.name)
            };

            let mut args: Vec<String> = vec![
                "source".to_string(),
                vpath,  // Now `vpath` is owned inside `args`
                "&&".to_string(),
                "uv".to_string(),
                "pip".to_string(),
                "install".to_string(),
            ];

            args.push(pkgs.join(" ")); // Store the owned String
            println!("Installing packages: {:?}", args);
            let agr_str = args.iter().map(String::as_str).collect::<Vec<_>>().join(" ");
            let mut child2 = utils::create_child_cmd("bash", &["-c", &agr_str]);

            utils::run_command(&mut child2).await;
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
