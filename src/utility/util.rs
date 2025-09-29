use colored::Colorize;
use tokio::fs;

use crate::{
    cmd::utils::exit_with_error,
    cmd::venv::{self, Venv},
    utility::util,
    venvmngr,
};

fn get_index(size: usize) -> Option<usize> {
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
        .or_else(|| exit_with_error("Error, please provide a valid index"))
}

pub async fn find_venv(
    name_pos: Option<String>,
    name: Option<String>,
    method: &str,
) -> Option<Venv> {
    let venv = match name.or(name_pos) {
        Some(n) => venv::Venv::new(n, "".to_string(), vec![], false),
        None => {
            let venvs = venvmngr::VENVMANAGER.list(Some(false)).await;
            if venvs.is_empty() {
                println!("{}", "No virtual environments found".yellow());
                return None;
            }
            for (i, venv) in venvs.iter().enumerate() {
                println!("{}. {}", (i + 1).to_string().cyan(), venv.green());
            }
            println!(
                "{} {}{}",
                "Please select a virtual environment to".cyan(),
                method.yellow(),
                " (c to cancel):".cyan()
            );
            match util::get_index(venvs.len()) {
                None => return None,
                Some(index) => {
                    venv::Venv::new(venvs[index - 1].clone(), "".to_string(), vec![], false)
                }
            }
        }
    };
    Some(venv)
}

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
            eprintln!(
                "{}",
                format!("Error reading requirements file: {}", e).red()
            );
            vec![]
        }
    }
}
