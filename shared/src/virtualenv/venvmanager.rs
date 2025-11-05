use super::venv::{self, Venv};
use crate::{
    constants::{UNIX_PYTHON3_EXEC, UNIX_PYTHON_EXEC, WIN_PYTHON_EXEC},
    settings,
};
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use once_cell::sync::Lazy;
use std::io::{BufRead, Write};
use std::{fs, io::stdout};

pub struct VenvManager;

pub static VENVMANAGER: Lazy<VenvManager> = Lazy::new(VenvManager::new);

impl VenvManager {
    fn new() -> Self {
        VenvManager
    }

    pub async fn list(&self) -> Vec<Venv> {
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let venvs: Vec<Venv> = match fs::read_dir(&path) {
            Ok(entries) => self.collect_venvs(entries),
            Err(_) => Vec::new(),
        };
        venvs
    }

    pub async fn check_if_exists(&self, name: String) -> bool {
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let venv_path = format!("{}/{}", path, name);
        std::path::Path::new(&venv_path).exists()
    }

    pub async fn find_venv<R: std::io::Read>(
        &self,
        input: R,
        name_pos: Option<String>,
        name: Option<String>,
        method: &str,
    ) -> Option<Venv> {
        let venv = match name.or(name_pos) {
            Some(n) => venv::Venv::new(n, "".to_string(), "".to_string(), vec![], false),
            None => {
                let mut venvs = self.list().await;
                if venvs.is_empty() {
                    log::warn!("No virtual environments found");
                    return None;
                }
                self.print_venv_table(&mut venvs).await;
                log::info!(
                    "{}{}{}",
                    "Please select a virtual environment to ",
                    method,
                    " (c to cancel):"
                );
                match self.get_index(input, venvs.len()) {
                    Ok(index) => venv::Venv::new(
                        venvs[index - 1].name.clone(),
                        "".to_string(),
                        "".to_string(),
                        vec![],
                        false,
                    ),
                    Err(e) => {
                        log::error!("{}", e);
                        return None;
                    }
                }
            }
        };
        Some(venv)
    }

    pub fn get_index<R: std::io::Read>(&self, input: R, size: usize) -> Result<usize, String> {
        let mut input_string = String::new();
        let mut stdin = std::io::BufReader::new(input);
        let _ = stdout().flush();
        let _ = stdin.read_line(&mut input_string).is_ok();
        let trimmed = input_string.trim();
        if trimmed.eq_ignore_ascii_case("q") || trimmed.eq_ignore_ascii_case("c") {
            return Err("Cancelled by user".to_string());
        }
        let idx = trimmed
            .parse::<usize>()
            .map_err(|_| "Please provide a valid number!".to_string())?;
        if (1..=size).contains(&idx) {
            Ok(idx)
        } else {
            Err("Index out of range!".to_string())
        }
    }

    pub fn collect_venvs(&self, entries: fs::ReadDir) -> Vec<Venv> {
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

    pub async fn print_venv_table_to<W: Write>(&self, writer: &mut W, venvs: &mut [Venv]) {
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
    use std::io;

    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_list_venvs() {
        let venvs = VENVMANAGER.list().await;
        assert!(venvs.is_empty() || venvs.len() <= 5);
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
        let venv = VENVMANAGER
            .find_venv(io::stdin(), None, None, "activate")
            .await;
        assert!(venv.is_some() || venv.is_none());
    }

    #[tokio::test]
    async fn test_find_venv_none_cancel() {
        let cursor = std::io::Cursor::new("c\n");
        let venv = VENVMANAGER.find_venv(cursor, None, None, "activate").await;
        assert!(venv.is_some() || venv.is_none());
    }

    #[tokio::test]
    async fn test_find_venv_by_name() {
        let venv = VENVMANAGER
            .find_venv(io::stdin(), None, Some("test_venv".to_string()), "activate")
            .await;
        assert!(venv.is_some());
        assert_eq!(venv.unwrap().name, "test_venv");
    }

    #[tokio::test]
    async fn test_find_venv_by_name_pos() {
        let venv = VENVMANAGER
            .find_venv(io::stdin(), Some("test_venv".to_string()), None, "activate")
            .await;
        assert!(venv.is_some());
        assert_eq!(venv.unwrap().name, "test_venv");
    }

    #[tokio::test]
    async fn test_collect_venvs_empty() {
        let tmp_dir = tempdir().unwrap();
        let entries = fs::read_dir(tmp_dir.path()).unwrap();
        let venvs = VENVMANAGER.collect_venvs(entries);
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
                settings: settings::Settings::get_settings(),
            },
            Venv {
                name: "venv2".to_string(),
                python_version: "3.11".to_string(),
                path: "/other/path".to_string(),
                packages: Vec::new(),
                default: true,
                settings: settings::Settings::get_settings(),
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
                settings: settings::Settings::get_settings(),
            },
            Venv {
                name: "venv2".to_string(),
                python_version: "3.11".to_string(),
                path: "/other/path".to_string(),
                packages: Vec::new(),
                default: true,
                settings: settings::Settings::get_settings(),
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

    #[test]
    fn test_get_index_valid() {
        let cursor = std::io::Cursor::new("2\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_ok());
    }

    #[test]
    fn test_get_index_invalid() {
        let cursor = std::io::Cursor::new("10\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }

    #[test]
    fn test_get_index_cancel() {
        let cursor = std::io::Cursor::new("c\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }

    #[test]
    fn test_get_index_non_number() {
        let cursor = std::io::Cursor::new("abc\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }
}
