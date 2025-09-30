use std::fs;

use colored::Colorize;
use once_cell::sync::Lazy;

use super::venv::{self, Venv};
use crate::cfg::settings;
use crate::shell::processes;
use crate::utility::constants::{UNIX_PYTHON3_EXEC, UNIX_PYTHON_EXEC, WIN_PYTHON_EXEC};
use crate::utility::util;

pub struct VenvManager;

pub static VENVMANAGER: Lazy<VenvManager> = Lazy::new(VenvManager::new);

impl VenvManager {
    fn new() -> Self {
        VenvManager
    }

    pub async fn list(&self) -> Vec<Venv> {
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let venvs: Vec<Venv> = match fs::read_dir(&path) {
            Ok(entries) => {
                let venvs: Vec<Venv> = entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| {
                        if entry.file_type().ok()?.is_dir() {
                            let dir_path = entry.path();
                            let python_paths = [
                                dir_path.join(WIN_PYTHON_EXEC),
                                dir_path.join(UNIX_PYTHON_EXEC),
                                dir_path.join(UNIX_PYTHON3_EXEC),
                            ];
                            if python_paths.iter().any(|p| p.exists()) {
                                Some(Venv::new(
                                    entry.file_name().to_str()?.to_string(),
                                    dir_path.to_str()?.to_string(),
                                    "".to_string(),
                                    vec![],
                                    false,
                                ))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                venvs
            }
            Err(_) => Vec::new(),
        };
        venvs
    }

    pub async fn check_if_exists(&self, name: String) -> bool {
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let venv_path = format!("{}/{}", path, name);
        std::path::Path::new(&venv_path).exists()
    }

    pub async fn find_venv(
        &self,
        name_pos: Option<String>,
        name: Option<String>,
        method: &str,
    ) -> Option<Venv> {
        let venv = match name.or(name_pos) {
            Some(n) => venv::Venv::new(n, "".to_string(), "".to_string(), vec![], false),
            None => {
                let mut venvs = self.list().await;
                if venvs.is_empty() {
                    println!("{}", "No virtual environments found".yellow());
                    return None;
                }
                util::print_venv_table(&mut venvs).await;
                println!(
                    "{} {}{}",
                    "Please select a virtual environment to".cyan(),
                    method.yellow(),
                    " (c to cancel):".cyan()
                );
                match self.get_index(venvs.len()) {
                    None => return None,
                    Some(index) => venv::Venv::new(
                        venvs[index - 1].name.clone(),
                        "".to_string(),
                        "".to_string(),
                        vec![],
                        false,
                    ),
                }
            }
        };
        Some(venv)
    }

    fn get_index(&self, size: usize) -> Option<usize> {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("q") || trimmed.eq_ignore_ascii_case("c") {
            return None;
        }
        trimmed
            .parse::<usize>()
            .ok()
            .filter(|&i| (1..=size).contains(&i))
            .or_else(|| processes::exit_with_error("Error, please provide a valid index"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_venvs() {
        let venvs = VENVMANAGER.list().await;
        // Assuming there are no virtual environments for the test
        assert!(venvs.is_empty());
    }

    #[tokio::test]
    async fn test_check_if_exists() {
        let exists = VENVMANAGER
            .check_if_exists("non_existent_venv".to_string())
            .await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_find_venv_none() {
        let venv = VENVMANAGER.find_venv(None, None, "activate").await;
        assert!(venv.is_none());
    }

    #[tokio::test]
    async fn test_find_venv_by_name() {
        let venv = VENVMANAGER
            .find_venv(None, Some("test_venv".to_string()), "activate")
            .await;
        assert!(venv.is_some());
        assert_eq!(venv.unwrap().name, "test_venv");
    }

    #[tokio::test]
    async fn test_find_venv_by_name_pos() {
        let venv = VENVMANAGER
            .find_venv(Some("test_venv".to_string()), None, "activate")
            .await;
        assert!(venv.is_some());
        assert_eq!(venv.unwrap().name, "test_venv");
    }
}
