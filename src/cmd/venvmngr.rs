use std::fs;

use colored::Colorize;
use once_cell::sync::Lazy;

use crate::cfg::settings;

pub struct VenvManager;

pub static VENVMANAGER: Lazy<VenvManager> = Lazy::new(|| VenvManager::new());

impl VenvManager {
    fn new() -> Self {
        VenvManager
    }

    pub async fn list(&self, print: Option<bool>) -> Vec<String> {
        let print = print.unwrap_or(true);
        if print {
            println!("{}", "Listing virtual environments".cyan());
        }
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let venvs: Vec<String> = match fs::read_dir(&path) {
            Ok(entries) => {
                let venvs: Vec<String> = entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| {
                        if entry.file_type().ok()?.is_dir() {
                            let dir_path = entry.path();
                            let python_paths = [
                                dir_path.join("Scripts").join("python.exe"),
                                dir_path.join("bin").join("python"),
                                dir_path.join("bin").join("python3"),
                            ];
                            if python_paths.iter().any(|p| p.exists()) {
                                entry.file_name().to_str().map(|s| s.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                if venvs.is_empty() && print {
                    println!("{}", "No virtual environments found".yellow());
                } else if print {
                    for venv in &venvs {
                        println!("{}", venv.green());
                    }
                }
                venvs
            }
            Err(_) => {
                if print {
                    println!("{}", "Error reading virtual environments directory".red());
                }
                Vec::new()
            }
        };
        venvs
    }
}
