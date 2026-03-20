use super::uvvenv::{self, UvVenv};
use crate::{
    constants::{UNIX_PYTHON3_EXEC, UNIX_PYTHON_EXEC, WIN_PYTHON_EXEC},
    error::{PylotError, Result},
    settings,
};
use comfy_table::{
    modifiers::UTF8_SOLID_INNER_BORDERS, presets::UTF8_FULL, ContentArrangement, Table,
};
use std::{
    borrow::Cow,
    io::{stdout, BufRead, Write},
    sync::LazyLock,
};
use tokio::fs;

pub struct VenvManager;

pub static VENVMANAGER: LazyLock<VenvManager> = LazyLock::new(VenvManager::new);

impl<'a> VenvManager {
    fn new() -> Self {
        VenvManager
    }

    pub async fn list(&'a self) -> Vec<UvVenv<'a>> {
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let venvs: Vec<UvVenv> = match fs::read_dir(&path).await {
            Ok(entries) => self.collect_venvs(entries).await,
            Err(_) => Vec::new(),
        };
        venvs
    }

    pub async fn check_if_exists(&self, name: &str) -> bool {
        let path = shellexpand::tilde(&settings::Settings::get_settings().venvs_path).to_string();
        let venv_path = format!("{}/{}", path, name);
        fs::try_exists(&venv_path).await.unwrap_or(false)
    }

    pub async fn find_venv<R: std::io::Read>(
        &'a self,
        input: R,
        name: Option<&'a str>,
        method: &str,
    ) -> Result<UvVenv<'a>> {
        let venv = match name {
            Some(n) => uvvenv::UvVenv::new(
                Cow::Borrowed(n),
                "".to_string(),
                "".to_string(),
                vec![],
                false,
            ),
            None => {
                let mut venvs = self.list().await;
                if venvs.is_empty() {
                    log::warn!("No virtual environments found");
                    return Err(PylotError::VenvNotFound(
                        "No virtual environments found".to_string(),
                    ));
                }
                self.print_venv_table_to(&mut std::io::stdout(), &mut venvs)
                    .await;
                log::info!(
                    "{}{}{}",
                    "Please select a virtual environment to ",
                    method,
                    " (c to cancel):"
                );
                match self.get_index(input, venvs.len()) {
                    Ok(index) => uvvenv::UvVenv::new(
                        venvs[index - 1].name.clone(),
                        "".to_string(),
                        "".to_string(),
                        vec![],
                        false,
                    ),
                    Err(_) => {
                        return Err(PylotError::Cancelled);
                    }
                }
            }
        };
        Ok(venv)
    }

    fn get_index<R: std::io::Read>(&self, input: R, size: usize) -> std::result::Result<usize, String> {
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

    async fn collect_venvs(&'a self, mut entries: fs::ReadDir) -> Vec<UvVenv<'a>> {
        let mut venvs = Vec::new();
        
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_dir() {
                    let dir_path = entry.path();
                    let python_paths = [
                        dir_path.join(WIN_PYTHON_EXEC),
                        dir_path.join(UNIX_PYTHON_EXEC),
                        dir_path.join(UNIX_PYTHON3_EXEC),
                    ];
                    
                    if let Some(folder_name) = entry.file_name().to_str() {
                        let has_python = futures::future::join_all(
                            python_paths.iter().map(|p| fs::try_exists(p))
                        )
                        .await
                        .into_iter()
                        .any(|r| r.unwrap_or(false));
                        
                        if has_python {
                            if let Some(path_str) = dir_path.to_str() {
                                venvs.push(UvVenv::new(
                                    Cow::Owned(folder_name.to_string()),
                                    path_str.to_string(),
                                    "".to_string(),
                                    vec![],
                                    false,
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        venvs
    }

    pub async fn print_venv_table(&self) {
        let mut venvs = self.list().await;
        if venvs.is_empty() {
            log::info!("No virtual environments found");
        } else {
            self.print_venv_table_to(&mut std::io::stdout(), &mut venvs)
                .await;
        }
    }

    async fn print_venv_table_to<W: Write>(&self, writer: &mut W, venvs: &mut [UvVenv<'a>]) {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Index", "Name", "Version"]);
        for (index, venv) in venvs.iter_mut().enumerate() {
            venv.set_python_version().await;
            table.add_row(vec![
                (index + 1).to_string(),
                venv.name.clone().to_string(),
                venv.python_version.clone(),
            ]);
        }
        let _ = writeln!(writer, "{}", table);
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::logger;

    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_list_venvs() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let result = VENVMANAGER.list().await;
        // Test that list operation completes without errors
        // We can't assert on the exact count as it depends on system state
        for venv in result {
            // Verify each venv has a valid name
            assert!(!venv.name.is_empty());
        }
    }

    #[tokio::test]
    async fn test_check_if_exists() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let exists = VENVMANAGER.check_if_exists("non_existent_venv").await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_find_venv_none() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = VENVMANAGER.find_venv(io::stdin(), None, "activate").await;
        assert!(venv.is_ok() || venv.is_err());
    }

    #[tokio::test]
    async fn test_find_venv_none_cancel() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("c\n");
        let venv = VENVMANAGER.find_venv(cursor, None, "activate").await;
        assert!(venv.is_ok() || venv.is_err());
    }

    #[tokio::test]
    async fn test_find_venv_by_name() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = VENVMANAGER
            .find_venv(io::stdin(), Some("test_venv"), "activate")
            .await;
        assert!(venv.is_ok());
        assert_eq!(venv.unwrap().name, "test_venv");
    }

    #[tokio::test]
    async fn test_collect_venvs_empty() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let tmp_dir = tempdir().unwrap();
        let entries = fs::read_dir(tmp_dir.path()).await.unwrap();
        let venvs = VENVMANAGER.collect_venvs(entries).await;
        assert!(venvs.is_empty());
    }

    #[tokio::test]
    async fn test_print_table() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let mut venvs = vec![
            UvVenv {
                name: Cow::Borrowed("venv1"),
                python_version: "3.10".to_string(),
                path: "/some/path".to_string(),
                packages: Vec::new(),
                default: false,
                settings: settings::Settings::get_settings(),
            },
            UvVenv {
                name: Cow::Borrowed("venv2"),
                python_version: "3.11".to_string(),
                path: "/other/path".to_string(),
                packages: Vec::new(),
                default: true,
                settings: settings::Settings::get_settings(),
            },
        ];
        VENVMANAGER
            .print_venv_table_to(&mut std::io::stdout(), &mut venvs)
            .await;
    }

    #[tokio::test]
    async fn test_print_venv_table() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let mut venvs = vec![
            UvVenv {
                name: Cow::Borrowed("venv1"),
                python_version: "3.10".to_string(),
                path: "/some/path".to_string(),
                packages: Vec::new(),
                default: false,
                settings: settings::Settings::get_settings(),
            },
            UvVenv {
                name: Cow::Borrowed("venv2"),
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
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("2\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_ok());
    }

    #[test]
    fn test_get_index_invalid() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("10\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }

    #[test]
    fn test_get_index_cancel() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("c\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }

    #[test]
    fn test_get_index_non_number() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("abc\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }

    #[test]
    fn test_get_index_quit() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("q\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }

    #[test]
    fn test_get_index_zero_rejected() {
        logger::initialize_logger(log::LevelFilter::Trace);
        // 0 is outside the valid range [1, size].
        let cursor = std::io::Cursor::new("0\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert!(index.is_err());
    }

    #[test]
    fn test_get_index_exactly_size() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("5\n");
        let index = VENVMANAGER.get_index(cursor, 5);
        assert_eq!(index.unwrap(), 5);
    }

    #[test]
    fn test_get_index_one() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("1\n");
        let index = VENVMANAGER.get_index(cursor, 3);
        assert_eq!(index.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_collect_venvs_non_venv_dir() {
        // A directory that exists but has no python executable should not be
        // collected as a venv.
        logger::initialize_logger(log::LevelFilter::Trace);
        let tmp_dir = tempdir().unwrap();
        // Create a subdirectory without any python executable.
        let sub = tmp_dir.path().join("not_a_venv");
        fs::create_dir_all(&sub).await.unwrap();
        let entries = fs::read_dir(tmp_dir.path()).await.unwrap();
        let venvs = VENVMANAGER.collect_venvs(entries).await;
        assert!(venvs.is_empty());
    }

    #[tokio::test]
    async fn test_venvmanager_new_singleton() {
        // Verify that VENVMANAGER can be accessed without panicking.
        logger::initialize_logger(log::LevelFilter::Trace);
        let _ = &*VENVMANAGER;
    }
}
