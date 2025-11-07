use crate::{
    constants::{DEFAULT_VENV_HOME, ERROR_VENV_NOT_EXISTS, POWERSHELL_CMD, PWSH_CMD, SH_CMD},
    processes, settings, utils, uvctrl,
    venvtraits::{Activate, Create, Delete},
};
use colored::Colorize;
use std::fs;
use tokio::fs as async_fs;

pub struct UvVenv {
    pub name: String,
    pub path: String,
    pub python_version: String,
    pub packages: Vec<String>,
    pub default: bool,
    pub settings: settings::Settings,
}

impl Create for UvVenv {
    async fn create(&self) -> Result<(), String> {
        if let Some((pwd, args)) = self.get_pwd_args() {
            let mut child = processes::create_child_cmd("uv", &args, "");
            let result = processes::run_command(&mut child).await;
            if result.is_err() {
                std::env::set_current_dir(pwd).unwrap();
                return Err(format!("{}", result.err().unwrap()));
            }
            let mut pkgs = self.packages.clone();
            if self.default {
                let default_pkgs = self.settings.default_pkgs.clone();
                pkgs.extend(default_pkgs);
            }
            if !pkgs.is_empty() {
                let venv_path = shellexpand::tilde(&self.settings.venvs_path).to_string();

                let (cmd, run, agr_str) = self.generate_command(pkgs, venv_path).await;
                let mut child2 = processes::create_child_cmd(cmd, &[&agr_str], run);

                processes::run_command(&mut child2)
                    .await
                    .map_err(|_| "Error installing packages".to_string())?;
            }
            std::env::set_current_dir(pwd).unwrap();
            Ok(())
        } else {
            Err("Error getting current directory".to_string())
        }
    }
}

impl Delete for UvVenv {
    async fn delete<R: std::io::Read>(&self, input: R, confirm: bool) {
        let path = shellexpand::tilde(&self.settings.venvs_path).to_string();
        let venv_path = format!("{}/{}", path, self.name);
        if !std::path::Path::new(&venv_path).exists() {
            log::error!("{}", ERROR_VENV_NOT_EXISTS);
            return;
        }
        let mut choice = !confirm;
        if confirm {
            log::info!(
                "{} {} {} {}",
                "Deleting virtual environment:",
                self.name.red(),
                "at".green(),
                venv_path.replace("\\", "/").red()
            );
            choice = utils::confirm(input);
        }
        if !choice {
            return;
        }
        match fs::remove_dir_all(venv_path) {
            Ok(_) => {
                if confirm {
                    log::info!("'{}' {}", self.name, "has been deleted")
                }
            }
            Err(e) => log::error!("{} {}", e, self.name),
        }
    }
}

impl Activate for UvVenv {
    async fn activate(&self) {
        let (shell, cmd, path) = self.get_shell_cmd();
        if !std::path::Path::new(&path).exists() {
            log::error!("{}", ERROR_VENV_NOT_EXISTS);
            return;
        }
        log::info!("\nActivating virtual environment: {}", self.name);
        log::warn!(
            "{} {}",
            "Note: To exit the virtual environment, type",
            "'exit'".green()
        );
        let _ = processes::activate_venv_shell(shell.as_str(), cmd);
    }
}

impl UvVenv {
    pub fn new(
        name: String,
        path: String,
        python_version: String,
        packages: Vec<String>,
        default: bool,
    ) -> Self {
        UvVenv {
            name,
            path,
            python_version,
            packages,
            default,
            settings: settings::Settings::get_settings(),
        }
    }

    pub async fn set_python_version(&mut self) {
        let cfg_path = format!("{}/pyvenv.cfg", self.path);
        if !async_fs::try_exists(&cfg_path).await.unwrap_or(false) {
            return;
        }
        if let Ok(content) = tokio::fs::read_to_string(cfg_path).await {
            for line in content.lines() {
                if line.starts_with("version") {
                    let parts: Vec<&str> = line.split('=').collect();
                    if parts.len() == 2 {
                        self.python_version = parts[1].trim().to_string();
                    }
                }
            }
        }
    }

    pub fn get_pwd_args(&self) -> Option<(std::path::PathBuf, [&str; 4])> {
        let pwd = std::env::current_dir().unwrap();
        let venvs_path = if self.settings.venvs_path.is_empty() {
            DEFAULT_VENV_HOME
        } else {
            &self.settings.venvs_path
        };
        let path = shellexpand::tilde(venvs_path).to_string();
        std::fs::create_dir_all(&path).unwrap();
        std::env::set_current_dir(&path).unwrap();
        let args = [
            "venv",
            self.name.as_str(),
            "--python",
            self.python_version.as_str(),
        ];
        log::info!("Creating virtual environment: {}", self.name);
        Some((pwd, args))
    }

    pub async fn generate_command(
        &self,
        pkgs: Vec<String>,
        venv_path: String,
    ) -> (&str, &'static str, String) {
        let (cmd, vcmd, run) = if cfg!(target_os = "windows") {
            let pwsh_cmd = if uvctrl::check(PWSH_CMD).await.is_ok() {
                PWSH_CMD
            } else {
                POWERSHELL_CMD
            };
            let venv_cmd = format!("{}/{}/scripts/activate.ps1", venv_path, self.name);
            (pwsh_cmd, venv_cmd, "-Command")
        } else {
            let venv_cmd = format!("{}/{}/bin/activate", venv_path, self.name);
            (SH_CMD, venv_cmd, "-c")
        };

        let mut args: Vec<String> = vec![
            vcmd,
            "&&".to_string(),
            "uv".to_string(),
            "pip".to_string(),
            "install".to_string(),
        ];
        if !cfg!(target_os = "windows") {
            args.insert(0, ".".to_string());
        }
        args.push(pkgs.join(" "));
        log::info!("{} {}", "Installing package(s):", pkgs.join(", "));
        let agr_str = args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        (cmd, run, agr_str)
    }

    fn get_shell_cmd(&self) -> (String, Vec<String>, String) {
        let path = shellexpand::tilde(&self.settings.venvs_path).to_string();
        let shell = processes::get_parent_shell();
        let (cmd, path) = if cfg!(target_os = "windows") {
            let venv_path = format!("{}/{}/scripts/activate.ps1", path, self.name);
            let venv_cmd = format!("{} && {}", venv_path, shell.as_str());
            (vec![venv_cmd], venv_path)
        } else {
            let venv_path = format!("{}/{}/bin/activate", path, self.name);
            let venv_cmd = format!(". {} && {} -i", venv_path, shell.as_str());
            (vec!["-c".to_string(), venv_cmd], venv_path)
        };
        (shell, cmd, path)
    }
}
