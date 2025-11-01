use std::io;

use colored::Colorize;
use shared::venvmanager;
use shared::{constants::ERROR_CREATING_VENV, utils, uv, venv};

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
    mut packages: Vec<String>,
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
    match update_packages_from_requirements(requirements, &mut packages).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "{}",
                format!("Error reading requirements file: {}", e).red()
            );
            return;
        }
    }
    let venv = venv::Venv::new(name, "".to_string(), python_version, packages, default);
    if let Err(e) = venv.create().await {
        eprintln!("{}", format!("{}: {}", ERROR_CREATING_VENV, e).red());
        venv.delete(io::stdin(), false).await;
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

pub async fn install<R: std::io::Read>(input: R) {
    if uv::check().await {
        println!("{}", "Astral UV is already installed.".yellow());
        return;
    }
    if let Err(e) = uv::install(input).await {
        eprintln!("{}", format!("Error installing Astral UV: {}", e).red());
    }
}

pub async fn update() {
    if uv::check().await {
        uv::update().await.unwrap_or_else(|e| {
            eprintln!("{}", format!("Error updating Astral UV: {}", e).red());
        });
    } else {
        eprintln!("{}", "Astral UV is not installed.".red());
    }
}

pub async fn uninstall<R: std::io::Read>(input: R) {
    if !uv::check().await {
        eprintln!("{}", "Astral UV is not installed.".yellow());
        return;
    }
    if let Err(e) = uv::uninstall(input).await {
        eprintln!("{}", format!("Error uninstalling Astral UV: {}", e).red());
    }
}

pub async fn list() {
    let venvs = venvmanager::VENVMANAGER.list().await;
    print_venvs(venvs).await;
}

async fn print_venvs(mut venvs: Vec<venv::Venv>) {
    if venvs.is_empty() {
        println!("{}", "No virtual environments found".yellow());
    } else {
        venvmanager::VENVMANAGER.print_venv_table(&mut venvs).await;
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use tokio::fs::write;

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
        create(None, None, "3.8".to_string(), vec![], "".to_string(), false).await;
    }

    #[tokio::test]
    async fn test_create_missing_uv() {
        let cursor = std::io::Cursor::new("y\n");
        uninstall(cursor.clone()).await;
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

    #[tokio::test]
    async fn update_packages_from_requirements_test() {
        let requirements = "test_requirements.txt".to_string();
        let mut packages = vec!["numpy".to_string()];
        let _ = write(&requirements, "pandas\nscipy\n").await;
        match update_packages_from_requirements(requirements.clone(), &mut packages).await {
            Ok(_) => {}
            Err(e) => {
                panic!("Error reading requirements file: {}", e);
            }
        }
        assert!(packages.contains(&"numpy".to_string()));
        assert!(packages.contains(&"pandas".to_string()));
        assert!(packages.contains(&"scipy".to_string()));
        std::fs::remove_file(requirements).unwrap();
    }

    #[tokio::test]
    async fn test_install_uv_no() {
        let cursor = std::io::Cursor::new("n\n");
        install(cursor).await;
    }

    #[tokio::test]
    async fn test_uninstall_uv_no() {
        let cursor = std::io::Cursor::new("n\n");
        uninstall(cursor).await;
    }

    #[tokio::test]
    async fn test_install_uv_yes() {
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            install(cursor.clone()).await;
            install(cursor.clone()).await;
            assert!(uv::check().await);
        }
        #[cfg(not(unix))]
        {
            let cursor = std::io::Cursor::new("y\n");
            install(cursor).await;
        }
    }

    #[tokio::test]
    async fn test_uninstall_uv_yes() {
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            install(cursor.clone()).await;
            assert!(uv::check().await);
            uninstall(cursor).await;
            assert!(!uv::check().await);
        }
        #[cfg(not(unix))]
        {
            let cursor = std::io::Cursor::new("y\n");
            uninstall(cursor).await;
        }
    }
}
