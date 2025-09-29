use std::io;

use colored::Colorize;

use crate::{
    cmd::{
        manage::{check, install, uninstall},
        utils, venv, venvmngr,
    },
    utility::util,
};

pub async fn run_activate(name_pos: Option<String>, name: Option<String>) {
    let venv = util::find_venv(name_pos, name, "activate").await;
    if let Some(v) = venv {
        v.activate().await
    }
}

pub async fn run_check() {
    println!(
        "{}",
        "Checking if Astral UV is installed and configured...".cyan()
    );
    if check().await {
        println!("{}", "Astral UV is installed".green());
        return;
    }
    println!("{}", "Astral UV was not found".red());
}

pub async fn run_create(
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
            utils::exit_with_error("Missing name for the environment.");
        }
    };
    if !check().await {
        utils::exit_with_error(
            "Astral UV is not installed. Please run 'uv install' to install it.",
        );
    }
    if venvmngr::VENVMANAGER.check_if_exists(name.clone()).await {
        utils::exit_with_error("Virtual environment with this name already exists.");
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
    let venv = venv::Venv::new(name, python_version, packages, default);
    if let Err(e) = venv.create().await {
        eprintln!(
            "{}",
            format!("Error creating virtual environment: {}", e).red()
        );
        venv.delete(false).await;
    }
}

pub async fn run_delete(name_pos: Option<String>, name: Option<String>) {
    let venv = util::find_venv(name_pos, name, "delete").await;
    if let Some(v) = venv {
        v.delete(true).await
    }
}

pub async fn run_install(update: bool) {
    if check().await && !update {
        println!("{}", "Astral UV is already installed.".yellow());
        return;
    }
    if let Err(e) = install(io::stdin()).await {
        eprintln!("{}", format!("Error installing Astral UV: {}", e).red());
    }
}

pub async fn run_uninstall() {
    if !check().await {
        utils::exit_with_error("Astral UV is not installed.");
    }
    if let Err(e) = uninstall(io::stdin()).await {
        eprintln!("{}", format!("Error uninstalling Astral UV: {}", e).red());
    }
}
