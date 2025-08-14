use crate::venvmngr;
use colored::Colorize;

use crate::{
    cmd::{
        utils,
        venv::{self, Venv},
    },
    utility::util,
};

fn get_index(size: usize) -> Option<usize> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("q") {
        return None;
    }
    trimmed
        .parse::<usize>()
        .ok()
        .filter(|&i| (1..=size).contains(&i))
        .or_else(|| utils::exit_with_error("Error, please provide a valid index"))
}

pub async fn find_venv(
    name_pos: Option<String>,
    name: Option<String>,
    method: String,
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
                println!("{}. {}", i + 1, venv);
            }
            println!("Please select a virtual environment to {}:", method);
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
