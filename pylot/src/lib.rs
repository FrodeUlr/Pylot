pub mod cli;

use std::io;

use shared::venvmanager;
use shared::{constants::ERROR_CREATING_VENV, utils, uvctrl, venv};

pub async fn activate(name_pos: Option<String>, name: Option<String>) {
    let venv = venvmanager::VENVMANAGER
        .find_venv(io::stdin(), name_pos, name, "activate")
        .await;
    if let Some(v) = venv {
        v.activate().await
    }
}

pub async fn check() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Checking if Astral UV is installed and configured...");
    match uvctrl::check("uv").await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn create(
    name_pos: Option<String>,
    name: Option<String>,
    python_version: String,
    mut packages: Vec<String>,
    requirements: String,
    default: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = match name.or(name_pos) {
        Some(n) => n,
        None => {
            return Err("Missing 'name' for the environment.".into());
        }
    };
    match uvctrl::check("uv").await {
        Ok(_) => {}
        Err(_) => {
            return Err(format!(
                "Astral UV is not installed. Please run '{} uv install' to install it.",
                env!("CARGO_PKG_NAME")
            )
            .into())
        }
    };
    if venvmanager::VENVMANAGER.check_if_exists(name.clone()).await {
        log::error!(
            "A virtual environment with the name '{}' already exists.",
            name
        );
        return Err("Venv already exists".into());
    }
    match update_packages_from_requirements(requirements, &mut packages).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error reading requirements file: {}", e);
            return Err(format!("Error reading requirements file: {}", e).into());
        }
    }
    let venv = venv::Venv::new(name, "".to_string(), python_version, packages, default);
    match venv.create().await {
        Ok(_) => Ok(()),
        Err(e) => {
            venv.delete(io::stdin(), false).await;
            Err(format!("{}: {}", ERROR_CREATING_VENV, e).into())
        }
    }
}

pub async fn update_packages_from_requirements(
    requirements: String,
    packages: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !requirements.is_empty() {
        match utils::read_requirements_file(&requirements).await {
            Ok(read_pkgs) => {
                for req in read_pkgs {
                    if !packages.contains(&req) {
                        packages.push(req);
                    }
                }
            }
            Err(e) => Err(e)?,
        }
    }
    Ok(())
}

pub async fn delete<R: std::io::Read, F: std::io::Read>(
    input: R,
    find_input: F,
    name_pos: Option<String>,
    name: Option<String>,
) {
    let venv = venvmanager::VENVMANAGER
        .find_venv(find_input, name_pos, name, "delete")
        .await;
    if let Some(v) = venv {
        v.delete(input, true).await
    }
}

pub async fn install<R: std::io::Read>(input: R) -> Result<(), Box<dyn std::error::Error>> {
    if (uvctrl::check("uv").await).is_ok() {
        log::info!("Astral UV is already installed.");
        return Ok(());
    }
    match uvctrl::install(input).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn update() {
    if (uvctrl::check("uv").await).is_err() {
        log::info!("Astral UV is not installed.");
        return;
    }
    uvctrl::update().await.unwrap_or_else(|e| {
        log::error!("{}", e);
    });
}

pub async fn uninstall<R: std::io::Read>(input: R) -> Result<(), Box<dyn std::error::Error>> {
    if (uvctrl::check("uv").await).is_err() {
        log::info!("Astral UV is not installed.");
        return Ok(());
    }
    match uvctrl::uninstall(input).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn list() {
    let venvs = venvmanager::VENVMANAGER.list().await;
    print_venvs(venvs).await;
}

pub async fn print_venvs(mut venvs: Vec<venv::Venv>) {
    if venvs.is_empty() {
        log::info!("No virtual environments found");
    } else {
        venvmanager::VENVMANAGER.print_venv_table(&mut venvs).await;
    }
}
