pub mod cli;

use std::io;

use shared::{
    constants::ERROR_CREATING_VENV,
    utils, uvctrl, uvvenv, venvmanager,
    venvtraits::{Activate, Create, Delete},
};

/// Activate a virtual environment by named position or name
///
/// * Examples
///
/// activate(Some("test_env".to_string()), None).await;
///
/// activate(None, Some("test_env".to_string())).await;
pub async fn activate(name: Option<String>) {
    let venv = venvmanager::VENVMANAGER
        .find_venv(io::stdin(), name, "activate")
        .await;
    if let Some(v) = venv {
        v.activate().await
    }
}

/// Check if Astral UV is installed and configured
pub async fn check() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Checking if Astral UV is installed and configured...");
    match uvctrl::check("uv").await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Create a new virtual environment
///
/// * Examples
///
/// ```
/// use pylot::create;
/// // With named_pos:
/// create("test_env".to_string(), "3.8".to_string(), vec!["numpy".to_string(), "pandas".to_string()], "".to_string(), false);
/// // Install default packages defined in settings.toml:
/// create("test_env".to_string(), "3.8".to_string(), vec![], "".to_string(), true);
/// ```
pub async fn create(
    name: String,
    python_version: String,
    mut packages: Vec<String>,
    requirements: String,
    default: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if name.is_empty() {
        return Err("A valid name is required".into());
    }
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
        return Err(format!(
            "A virtual environment with the name {} already exists",
            name
        )
        .into());
    }
    match update_packages_from_requirements(requirements, &mut packages).await {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("Error reading requirements file: {}", e).into());
        }
    }
    let venv = uvvenv::UvVenv::new(name, "".to_string(), python_version, packages, default);
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
    name: Option<String>,
) {
    let venv = venvmanager::VENVMANAGER
        .find_venv(find_input, name, "delete")
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

pub async fn print_venvs(mut venvs: Vec<uvvenv::UvVenv>) {
    if venvs.is_empty() {
        log::info!("No virtual environments found");
    } else {
        venvmanager::VENVMANAGER.print_venv_table(&mut venvs).await;
    }
}
