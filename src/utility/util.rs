use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, ContentArrangement, Table};
use tokio::fs;

use crate::{cmd::utils::exit_with_error, cmd::venv::Venv};

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
    println!("{}", table);
}
