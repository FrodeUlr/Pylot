use crate::{core::venv::Venv, shell::processes::exit_with_error};
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use std::io::{stdout, BufRead, Write};
use tokio::fs;

pub async fn read_requirements_file(requirements: &str) -> Vec<String> {
    if !fs::try_exists(requirements).await.unwrap_or(false) {
        exit_with_error(&format!("Requirements file '{}' does not exist", requirements).red())
    }
    let content = tokio::fs::read_to_string(requirements).await;
    match content {
        Ok(c) => c
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect(),
        Err(e) => {
            exit_with_error(&format!("Error reading requirements file: {}", e));
        }
    }
}
pub async fn print_venv_table(venvs: &mut [Venv]) {
    print_venv_table_to(&mut std::io::stdout(), venvs).await;
}
async fn print_venv_table_to<W: Write>(writer: &mut W, venvs: &mut [Venv]) {
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

pub fn confirm<R: std::io::Read>(input: R) -> bool {
    let mut stdin = std::io::BufReader::new(input);
    print!("{}", "Do you want to continue? (y/n): ".cyan());
    let _ = stdout().flush();
    let mut input_string = String::new();
    if stdin.read_line(&mut input_string).is_ok() {
        matches!(input_string.trim(), "y" | "yes" | "Y" | "YES")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_requirements_file() {
        let test_file = "test_requirements.txt";
        let content = "package1\npackage2\n# This is a comment\n\npackage3\n";
        fs::write(test_file, content).await.unwrap();

        let packages = read_requirements_file(test_file).await;
        assert_eq!(packages, vec!["package1", "package2", "package3"]);

        fs::remove_file(test_file).await.unwrap();
    }

    #[tokio::test]
    async fn test_read_requirements_file_nonexistent() {
        let test_file = "nonexistent_requirements.txt";
        let result = std::panic::catch_unwind(|| {
            let _ = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(read_requirements_file(test_file));
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_confirm_yes() {
        let cursor = std::io::Cursor::new("y\n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_no() {
        let cursor = std::io::Cursor::new("n\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_invalid() {
        let cursor = std::io::Cursor::new("x\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_empty() {
        let cursor = std::io::Cursor::new("\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_whitespace() {
        let cursor = std::io::Cursor::new("   y   \n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_uppercase() {
        let cursor = std::io::Cursor::new("Y\n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_uppercase_yes() {
        let cursor = std::io::Cursor::new("YES\n");
        let result = confirm(cursor);
        assert!(result);
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
        print_venv_table(&mut venvs).await;
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
        print_venv_table_to(&mut output, &mut venvs).await;

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("venv1"));
        assert!(output_str.contains("3.10"));
        assert!(output_str.contains("venv2"));
        assert!(output_str.contains("3.11"));
    }
}
