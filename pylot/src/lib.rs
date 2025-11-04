use std::io;

use shared::venvmanager;
use shared::{constants::ERROR_CREATING_VENV, utils, uvctrl, venv};

pub async fn activate(name_pos: Option<String>, name: Option<String>) {
    let venv = venvmanager::VENVMANAGER
        .find_venv(name_pos, name, "activate")
        .await;
    if let Some(v) = venv {
        v.activate().await
    }
}

pub async fn check() {
    log::info!("Checking if Astral UV is installed and configured...");
    if uvctrl::check().await {
        log::info!("Astral UV is installed");
        return;
    }
    log::warn!("Astral UV was not found");
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
    if !uvctrl::check().await {
        log::error!(
            "Astral UV is not installed. Please run '{} uv install' to install it.",
            env!("CARGO_PKG_NAME")
        );
        return Err("Astral UV not installed".into());
    }
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

async fn update_packages_from_requirements(
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

pub async fn delete<R: std::io::Read>(input: R, name_pos: Option<String>, name: Option<String>) {
    let venv = venvmanager::VENVMANAGER
        .find_venv(name_pos, name, "delete")
        .await;
    if let Some(v) = venv {
        v.delete(input, true).await
    }
}

pub async fn install<R: std::io::Read>(input: R) -> Result<(), Box<dyn std::error::Error>> {
    if uvctrl::check().await {
        log::info!("Astral UV is already installed.");
        return Ok(());
    }
    match uvctrl::install(input).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn update() {
    if uvctrl::check().await {
        uvctrl::update().await.unwrap_or_else(|e| {
            log::error!("{}", e);
        });
    } else {
        log::error!("Astral UV is not installed.");
    }
}

pub async fn uninstall<R: std::io::Read>(input: R) -> Result<(), Box<dyn std::error::Error>> {
    if !uvctrl::check().await {
        log::error!("Astral UV is not installed");
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

async fn print_venvs(mut venvs: Vec<venv::Venv>) {
    if venvs.is_empty() {
        log::info!("No virtual environments found");
    } else {
        venvmanager::VENVMANAGER.print_venv_table(&mut venvs).await;
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use tokio::fs::{self, write};

    #[tokio::test]
    async fn test_check() {
        check().await;
    }

    #[tokio::test]
    async fn test_list() {
        list().await;
    }

    #[tokio::test]
    async fn test_print_venvs_empty() {
        print_venvs(vec![]).await;
    }

    #[tokio::test]
    async fn test_print_venvs_non_empty() {
        let venv = venv::Venv::new(
            "test_env".to_string(),
            "/path/to/test_env".to_string(),
            "3.8".to_string(),
            vec!["numpy".to_string()],
            false,
        );
        print_venvs(vec![venv]).await;
    }

    #[tokio::test]
    async fn test_delete() {
        delete(io::stdin(), Some("test_env".to_string()), None).await;
    }

    #[tokio::test]
    async fn test_activate() {
        activate(Some("test_env_not_here".to_string()), None).await;
    }

    #[tokio::test]
    async fn test_create_missing_name() {
        let result = create(None, None, "3.8".to_string(), vec![], "".to_string(), false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_missing_uv() {
        //only run on github agents
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        let cursor = std::io::Cursor::new("y\n");
        let result_un = uninstall(cursor).await;
        assert!(result_un.is_ok());
        let result = create(
            Some("test_env".to_string()),
            None,
            "3.8".to_string(),
            vec![],
            "".to_string(),
            false,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_packages_from_requirements_test() {
        let requirements = "test_requirements.txt".to_string();
        let mut packages = vec!["numpy".to_string()];
        let _ = write(&requirements, "pandas\nscipy\n").await;
        let result = update_packages_from_requirements(requirements.clone(), &mut packages).await;
        assert!(result.is_ok());
        assert!(packages.contains(&"numpy".to_string()));
        assert!(packages.contains(&"pandas".to_string()));
        assert!(packages.contains(&"scipy".to_string()));
        fs::remove_file(requirements).await.unwrap();
    }

    #[tokio::test]
    async fn test_install_uv_no() {
        let cursor = std::io::Cursor::new("n\n");
        let result_in = install(cursor.clone()).await;
        assert!(result_in.is_ok());
    }

    #[tokio::test]
    async fn test_uninstall_uv_no() {
        let cursor = std::io::Cursor::new("n\n");
        let result_un = uninstall(cursor).await;
        assert!(result_un.is_ok());
    }

    #[tokio::test]
    async fn test_install_uv_yes() {
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_in = install(cursor.clone()).await;
            assert!(result_in.is_ok());
        }
        #[cfg(not(unix))]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_in = install(cursor.clone()).await;
            assert!(result_in.is_ok());
        }
    }

    #[tokio::test]
    async fn test_uninstall_uv_yes() {
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_in = install(cursor.clone()).await;
            assert!(result_in.is_ok());
            let result_un = uninstall(cursor).await;
            assert!(result_un.is_ok());
        }
        #[cfg(not(unix))]
        {
            let cursor = std::io::Cursor::new("y\n");
            let result_un = uninstall(cursor).await;
            assert!(result_un.is_ok());
        }
    }

    #[tokio::test]
    async fn test_create_venv() {
        #[cfg(unix)]
        {
            use shellexpand::tilde;

            let cursor = std::io::Cursor::new("y\n");
            let result_in = install(cursor.clone()).await;
            assert!(result_in.is_ok());
            let uv_path = tilde("~/.local/bin/uv");
            std::env::set_var(
                "PATH",
                format!("{}:{}", uv_path, std::env::var("PATH").unwrap()),
            );
            let result = create(
                Some("test_env".to_string()),
                None,
                "3.8".to_string(),
                vec!["numpy".to_string()],
                "".to_string(),
                false,
            )
            .await;
            assert!(result.is_ok());
            delete(cursor.clone(), Some("test_env".to_string()), None).await;
            let result_un = uninstall(cursor).await;
            assert!(result_un.is_ok());
        }
    }
}
