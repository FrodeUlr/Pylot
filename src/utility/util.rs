use colored::Colorize;

use crate::{
    cmd::{
        utils,
        venvmgr::{self, Venv},
    },
    utility::util,
};

pub fn get_index(size: usize) -> usize {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|&i| (1..=size).contains(&i))
        .unwrap_or_else(|| utils::exit_with_error("Error, please provide a valid index"))
}

pub async fn find_venv(
    name_pos: Option<String>,
    name: Option<String>,
    method: String,
) -> Option<Venv> {
    let venv = match name.or(name_pos) {
        Some(n) => venvmgr::Venv::new(n, "".to_string(), vec![], false),
        None => {
            let venvs = venvmgr::Venv::list(Some(false)).await;
            if venvs.is_empty() {
                println!("{}", "No virtual environments found".yellow());
                return None;
            }
            for (i, venv) in venvs.iter().enumerate() {
                println!("{}. {}", i + 1, venv);
            }
            println!("Please select a virtual environment to {}:", method);
            let index = util::get_index(venvs.len());
            venvmgr::Venv::new(venvs[index - 1].clone(), "".to_string(), vec![], false)
        }
    };
    Some(venv)
}
