use colored::Colorize;
use shared::venvmanager;
use shared::{constants::ERROR_CREATING_VENV, processes, utils, uv, venv};
use std::io;

pub async fn activate(name_pos: Option<String>, name: Option<String>) {
    let venv = venvmanager::VENVMANAGER
        .find_venv(name_pos, name, "activate")
        .await;
    if let Some(v) = venv {
        v.activate().await
    }
}

pub async fn check() {
    println!(
        "{}",
        "Checking if Astral UV is installed and configured...".cyan()
    );
    if uv::check().await {
        println!("{}", "Astral UV is installed".green());
        return;
    }
    println!("{}", "Astral UV was not found".red());
}

pub async fn create(
    name_pos: Option<String>,
    name: Option<String>,
    python_version: String,
    packages: Vec<String>,
    requirements: String,
    default: bool,
) {
    let name = match name.or(name_pos) {
        Some(n) => n,
        None => {
            eprintln!("{}", "Error: Missing name for the environment.".red());
            return;
        }
    };
    if !uv::check().await {
        eprintln!(
            "{}",
            "Astral UV is not installed. Please run 'install-uv to install it.".red()
        );
        return;
    }
    if venvmanager::VENVMANAGER.check_if_exists(name.clone()).await {
        eprintln!(
            "{}",
            format!(
                "Error: A virtual environment with the name '{}' already exists.",
                name
            )
            .red()
        );
        return;
    }
    let mut packages = packages;
    if !requirements.is_empty() {
        let read_pkgs = utils::read_requirements_file(&requirements).await;
        for req in read_pkgs {
            if !packages.contains(&req) {
                packages.push(req);
            }
        }
    }
    let venv = venv::Venv::new(name, "".to_string(), python_version, packages, default);
    if let Err(e) = venv.create().await {
        eprintln!("{}", format!("{}: {}", ERROR_CREATING_VENV, e).red());
        venv.delete(false).await;
    }
}

pub async fn delete(name_pos: Option<String>, name: Option<String>) {
    let venv = venvmanager::VENVMANAGER
        .find_venv(name_pos, name, "delete")
        .await;
    if let Some(v) = venv {
        v.delete(true).await
    }
}

pub async fn install(update: bool) {
    if uv::check().await && !update {
        println!("{}", "Astral UV is already installed.".yellow());
        return;
    }
    if let Err(e) = uv::install(io::stdin()).await {
        eprintln!("{}", format!("Error installing Astral UV: {}", e).red());
    }
}

pub async fn uninstall() {
    if !uv::check().await {
        processes::exit_with_error("Astral UV is not installed.");
    }
    if let Err(e) = uv::uninstall(io::stdin()).await {
        eprintln!("{}", format!("Error uninstalling Astral UV: {}", e).red());
    }
}

pub async fn list() {
    let mut venvs = venvmanager::VENVMANAGER.list().await;
    if venvs.is_empty() {
        println!("{}", "No virtual environments found".yellow());
    } else {
        venvmanager::VENVMANAGER.print_venv_table(&mut venvs).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check() {
        check().await;
    }

    #[tokio::test]
    async fn test_list() {
        list().await;
    }

    #[tokio::test]
    async fn test_delete() {
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        delete(Some("test_env".to_string()), None).await;
    }

    #[tokio::test]
    async fn test_activate() {
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        activate(Some("test_env_not_here".to_string()), None).await;
    }

    #[tokio::test]
    async fn test_create_missing_name() {
        create(None, None, "3.8".to_string(), vec![], "".to_string(), false).await;
    }

    #[tokio::test]
    async fn test_create_missing_uv() {
        //only run on github agents
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        create(
            Some("test_env".to_string()),
            None,
            "3.8".to_string(),
            vec![],
            "".to_string(),
            false,
        )
        .await
    }
}
