use super::venv::{self, Venv};
use crate::{
    constants::{UNIX_PYTHON3_EXEC, UNIX_PYTHON_EXEC, WIN_PYTHON_EXEC},
    processes, settings,
};
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use once_cell::sync::Lazy;
use std::fs;
use std::io::Write;

pub struct VenvManager;

pub static VENVMANAGER: Lazy<VenvManager> = Lazy::new(VenvManager::new);

impl VenvManager {
    fn new() -> Self {
        VenvManager
    }

    pub async fn list(&self) -> Vec<Venv> {
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let venvs: Vec<Venv> = match fs::read_dir(&path) {
            Ok(entries) => Self::collect_venvs(entries),
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
                self.print_venv_table(&mut venvs).await;
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

    fn collect_venvs(entries: fs::ReadDir) -> Vec<Venv> {
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

    pub async fn print_venv_table(&self, venvs: &mut [Venv]) {
        self.print_venv_table_to(&mut std::io::stdout(), venvs)
            .await;
    }

    async fn print_venv_table_to<W: Write>(&self, writer: &mut W, venvs: &mut [Venv]) {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Index", "Name", "Version"]);
        for (index, venv) in venvs.iter_mut().enumerate() {
            venv.set_python_version().await;
            table.add_row(vec![
                (index + 1).to_string(),
                venv.name.clone(),
                venv.python_version.clone(),
            ]);
        }
        writeln!(writer, "{}", table).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_venvs() {
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        let venvs = VENVMANAGER.list().await;
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
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
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

    #[tokio::test]
    async fn test_collect_venvs_empty() {
        let entries = fs::read_dir("/non_existent_directory").unwrap_or_else(|_| {
            fs::read_dir(".").expect("Failed to read current directory for test")
        });
        let venvs = VenvManager::collect_venvs(entries);
        assert!(venvs.is_empty());
    }

    #[tokio::test]
    async fn test_print_table() {
        let mut venvs = vec![
            Venv {
                name: "venv1".to_string(),
                python_version: "3.10".to_string(),
                path: "/some/path".to_string(),
                packages: Vec::new(),
                default: false,
            },
            Venv {
                name: "venv2".to_string(),
                python_version: "3.11".to_string(),
                path: "/other/path".to_string(),
                packages: Vec::new(),
                default: true,
            },
        ];
        VENVMANAGER.print_venv_table(&mut venvs).await;
    }

    #[tokio::test]
    async fn test_print_venv_table() {
        let mut venvs = vec![
            Venv {
                name: "venv1".to_string(),
                python_version: "3.10".to_string(),
                path: "/some/path".to_string(),
                packages: Vec::new(),
                default: false,
            },
            Venv {
                name: "venv2".to_string(),
                python_version: "3.11".to_string(),
                path: "/other/path".to_string(),
                packages: Vec::new(),
                default: true,
            },
        ];

        let mut output = Vec::new();
        VENVMANAGER
            .print_venv_table_to(&mut output, &mut venvs)
            .await;

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("venv1"));
        assert!(output_str.contains("3.10"));
        assert!(output_str.contains("venv2"));
        assert!(output_str.contains("3.11"));
    }
}
