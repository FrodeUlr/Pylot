use colored::Colorize;

use super::utils;
use crate::cfg::settings;
use std::fs;

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
        println!("Creating virtual environment: {}", self.name.cyan());
        let mut child = utils::create_child_cmd("uv", args);
        utils::run_command(&mut child).await;
        let pkgs = self.packages.clone();
        if !self.packages.is_empty() && self.packages[0] != "[]" {
            let venn_path = shellexpand::tilde(&settings.venvs_path).to_string();

            let (cmd, vcmd, run) = if cfg!(target_os = "windows") {
                let venv_cmd = format!("{}/{}/scripts/activate.ps1", venn_path, self.name);
                ("pwsh", venv_cmd, "-Command")
            } else {
                let venv_cmd = format!("{}/{}/bin/activate", venn_path, self.name);
                ("bash", venv_cmd, "-c")
            };

            let mut args: Vec<String> = vec![
                vcmd,
                "&&".to_string(),
                "uv".to_string(),
                "pip".to_string(),
                "install".to_string(),
            ];
            if !cfg!(target_os = "windows") {
                args.insert(0, "source".to_string());
            }
            args.push(pkgs.join(" "));
            println!("{} {}", "Installing package(s):".cyan(), pkgs.join(", ").cyan());
            let agr_str = args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" ");
            let mut child2 = utils::create_child_cmd_run(cmd, run, &[&agr_str]);

            utils::run_command(&mut child2).await;
        }
        std::env::set_current_dir(pwd).unwrap();
    }

    pub async fn delete(&self) {
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let venv_path = format!("{}/{}", path, self.name);
        if !std::path::Path::new(&venv_path).exists() {
            println!("{}", "Virtual environment does not exist".yellow());
            return;
        }
        println!("{} {} {} {}", "Deleting virtual environment:".yellow(), self.name.red(), "at".yellow(), venv_path.replace("\\", "/").red());
        let choice = utils::confirm();
        if !choice {
            return;
        }
        match fs::remove_dir_all(venv_path) {
            Ok(_) => println!("{} {}", self.name.red(), "has been deleted".green()),
            Err(e) => println!("{} {}", e.to_string().red(), self.name),
        }
    }

    pub async fn list() {
        println!("{}", "Listing virtual environments".cyan());
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        match fs::read_dir(&path) {
            Ok(entries) => {
                let entries: Vec<_> = entries.filter_map(Result::ok).collect();
                if entries.is_empty() {
                    println!("{}", "No virtual environments found".yellow());
                    return;
                } else {
                    for entry in entries {
                        if entry.file_type().unwrap().is_dir() {
                            println!("{}", entry.file_name().to_str().unwrap().green());
                        }
                    }
                }
            }
            Err(_) => {
                println!("{}", "No virtual environments found".yellow());
            }
        }
    }

    pub async fn activate(&self) {
        println!("{} {}","Activating virtual environment:".cyan(), self.name.green());
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let shell = utils::get_parent_shell();
        let (cmd, path) = if cfg!(target_os = "windows") {
            let venv_path = format!("{}/{}/scripts/activate.ps1", path, self.name);
            let venv_cmd = format!("{} && {}", venv_path, shell.as_str());
            (vec!(venv_cmd), venv_path)
        } else {
            let venv_path = format!("{}/{}/bin/activate", path, self.name);
            let venv_cmd = format!("source {} && {} -i", venv_path, shell.as_str());
            (vec!("-c".to_string(), venv_cmd), venv_path)
        };
        if !std::path::Path::new(&path).exists() {
            println!("Virtual environment does not exist");
            return;
        }
        utils::activate_venv_shell(shell.as_str(), cmd);
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
        let venv = Venv::new(
            "test_venv_clean".to_string(),
            "3.9".to_string(),
            vec!["numpy".to_string(), "pandas".to_string()],
        );
        assert_eq!(venv.name, "test_venv_clean");
        assert_eq!(venv.python_version, "3.9");
        assert_eq![venv.packages, &["numpy", "pandas"]]
    }
}
