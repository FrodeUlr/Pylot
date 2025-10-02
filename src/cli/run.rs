use crate::{
    core::{uv, venv, venvmanager},
    shell::processes,
    utility::{constants::ERROR_CREATING_VENV, util},
};
use colored::Colorize;
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
            processes::exit_with_error("Missing name for the environment.");
        }
    };
    if !uv::check().await {
        processes::exit_with_error(
            "Astral UV is not installed. Please run 'uv install' to install it.",
        );
    }
    if venvmanager::VENVMANAGER.check_if_exists(name.clone()).await {
        processes::exit_with_error("Virtual environment with this name already exists.");
    }
    let mut packages = packages;
    if !requirements.is_empty() {
        let read_pkgs = util::read_requirements_file(&requirements).await;
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
        util::print_venv_table(&mut venvs).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check() {
        let is_installed = uv::check().await;
        if is_installed {
            println!("{}", "Astral UV is installed".green());
        } else {
            println!("{}", "Astral UV is not installed".red());
        }
    }

    #[tokio::test]
    async fn test_list() {
        list().await;
    }

    // #[tokio::test]
    // async fn test_create() {
    //     if std::env::var("GITHUB_ACTIONS").is_err() {
    //         println!("Skipping test in non-GitHub Actions environment");
    //         return;
    //     }
    //     create(
    //         Some("test_env".to_string()),
    //         None,
    //         "3.8".to_string(),
    //         vec!["requests".to_string()],
    //         "".to_string(),
    //         false,
    //     )
    //     .await;
    // }
    //
    // #[tokio::test]
    // async fn test_delete() {
    //     if std::env::var("GITHUB_ACTIONS").is_err() {
    //         println!("Skipping test in non-GitHub Actions environment");
    //         return;
    //     }
    //     delete(Some("test_env".to_string()), None).await;
    // }
    //
    // #[tokio::test]
    // async fn test_activate() {
    //     if std::env::var("GITHUB_ACTIONS").is_err() {
    //         println!("Skipping test in non-GitHub Actions environment");
    //         return;
    //     }
    //     activate(Some("test_env_not_here".to_string()), None).await;
    // }

    // #[tokio::test]
    // async fn test_install() {
    //     if std::env::var("GITHUB_ACTIONS").is_err() {
    //         println!("Skipping test in non-GitHub Actions environment");
    //         return;
    //     }
    //     install(false).await;
    // }
    //
    // #[tokio::test]
    // async fn test_uninstall() {
    //     if std::env::var("GITHUB_ACTIONS").is_err() {
    //         println!("Skipping test in non-GitHub Actions environment");
    //         return;
    //     }
    //     uninstall().await;
    // }
}
