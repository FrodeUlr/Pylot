//! A CLI to manage Python virtual environments using Astral UV
//!
//! [![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/FrodeUlr/pylot/rust.yml?branch=main&style=for-the-badge&logo=github)](https://github.com/FrodeUlr/Pylot) [![Codecov](https://img.shields.io/codecov/c/github/FrodeUlr/Pylot?style=for-the-badge&logo=codecov&label=CODECOV)](https://codecov.io/github/FrodeUlr/pylot)
//!
pub mod cli;

use std::{borrow::Cow, io};

use pylot_shared::{
    constants::{DEFAULT_PYTHON_VERSION, ERROR_CREATING_VENV},
    error::{PylotError, Result},
    utils, uvctrl, uvvenv, venvmanager,
    venvtraits::{Activate, Create, Delete},
};

/// Activate a virtual environment by named position or name
///
/// # Returns
/// * `Result<()>` - Ok if activated
///
/// # Examples
/// ```
/// use pylot::activate;
/// activate(Some("test_env"));
/// ```
pub async fn activate(name: Option<&str>) -> Result<()> {
    let venv = venvmanager::VENVMANAGER
        .find_venv(io::stdin(), name, "activate")
        .await?;
    venv.activate().await
}

/// Check if Astral UV is installed and configured
///
/// # Returns
/// * `Result<()>` - Ok if installed, Err if not
///
/// # Examples
/// ```
/// use pylot::check;
/// check();
/// ```
pub async fn check() -> Result<()> {
    log::info!("Checking if Astral UV is installed and configured...");
    uvctrl::check("uv")
        .await
        .map(|_| ())
        .map_err(|e| PylotError::Other(e.to_string()))
}

/// Create a new virtual environment
///
/// # Arguments
/// * `name` - The name of the virtual environment
/// * `python_version` - The Python version to use
/// * `packages` - A vector of packages to install
/// * `requirements` - A requirements file to install packages from
/// * `default` -  Whether to install default packages from settings.toml
///
/// # Returns
/// * `Result<()>` - Ok if created
///
/// # Examples
/// ```
/// use pylot::create;
///
/// // With named_pos:
/// let numpy = "numpy".to_string();
/// let pandas = "pandas".to_string();
/// create("test_env", Some("3.8"), Some(vec![numpy, pandas]), None, false);
/// // Install default packages defined in settings.toml:
/// create("test_env", Some("3.8"), None, None, true);
/// // With requirements file:
/// create("test_env", None, None, Some("requirements.txt"), false);
/// ```
pub async fn create(
    name: &str,
    python_version: Option<&str>,
    packages: Option<Vec<String>>,
    requirements: Option<&str>,
    default: bool,
) -> Result<()> {
    // Validate venv name
    uvvenv::UvVenv::validate_venv_name(name)?;

    uvctrl::check("uv").await.map_err(|_| {
        PylotError::Other(format!(
            "Astral UV is not installed. Please run '{} uv install' to install it.",
            env!("CARGO_PKG_NAME")
        ))
    })?;

    let mut pkgs = packages.unwrap_or_default();

    if venvmanager::VENVMANAGER.check_if_exists(name).await {
        return Err(PylotError::VenvExists(format!(
            "A virtual environment with the name {} already exists",
            name
        )));
    }

    if let Some(req) = requirements {
        update_packages_from_requirements(req, &mut pkgs).await?;
    }

    // Validate all package names
    for pkg in &pkgs {
        uvvenv::UvVenv::validate_package_name(pkg)?;
    }

    let venv = uvvenv::UvVenv::new(
        Cow::Borrowed(name),
        "".to_owned(),
        python_version.unwrap_or(DEFAULT_PYTHON_VERSION).to_owned(),
        pkgs,
        default,
    );

    match venv.create().await {
        Ok(_) => Ok(()),
        Err(e) => {
            // Try to clean up failed venv creation
            let _ = venv.delete(io::stdin(), false).await;
            Err(PylotError::Other(format!("{}: {}", ERROR_CREATING_VENV, e)))
        }
    }
}

async fn update_packages_from_requirements(
    requirements: &str,
    packages: &mut Vec<String>,
) -> Result<()> {
    if !requirements.is_empty() {
        let read_pkgs = utils::read_requirements_file(requirements)
            .await
            .map_err(|e| PylotError::Other(e.to_string()))?;

        // Preserve package order while deduplicating
        // This ensures installation order is maintained, which can matter
        // for packages with conflicting dependencies or when using --no-deps
        for req in read_pkgs {
            if !packages.contains(&req) {
                packages.push(req);
            }
        }
    }
    Ok(())
}

/// Delete a virtual environment by name or index position
///
/// # Arguments
/// * `confirm_input` - A reader for user input (e.g., stdin)
/// * `find_input` - A reader for user input to find the venv (e.g., stdin)
/// * `name` - The name of the virtual environment to delete
///
/// # Returns
/// * `Result<()>` - Ok if deleted
///
/// # Examples
/// ```
/// use pylot::delete;
/// use std::io;
///
/// // With name provided:
/// delete(io::stdin(), io::stdin(), Some("test_env"));
/// // Without name provided, will prompt user to select:
/// delete(io::stdin(), io::stdin(), None);
/// ```
pub async fn delete<R: std::io::Read, F: std::io::Read>(
    confirm_input: R,
    find_input: F,
    name: Option<&str>,
) -> Result<()> {
    let venv = venvmanager::VENVMANAGER
        .find_venv(find_input, name, "delete")
        .await?;

    venv.delete(confirm_input, true).await?;
    log::info!("Virtual environment '{}' deleted.", venv.name);
    Ok(())
}

/// Install Astral UV
///
/// # Arguments
/// * `input` - A reader for user input (e.g., stdin)
///
/// # Returns
/// * `Result<()>` - Ok if installed
///
/// # Examples
/// ```
/// use pylot::install;
/// use std::io;
///
/// install(io::stdin());
/// ```
pub async fn install<R: std::io::Read>(input: R) -> Result<()> {
    if (uvctrl::check("uv").await).is_ok() {
        log::info!("Astral UV is already installed.");
        return Ok(());
    }
    uvctrl::install(input).await.map_err(PylotError::Other)
}

/// Update Astral UV
/// Checks for updates and applies them if available
///
/// # Returns
/// * `()` - Nothing
///
/// # Examples
/// ```
/// use pylot::update;
///
/// update();
/// ```
pub async fn update() {
    if (uvctrl::check("uv").await).is_err() {
        log::info!("Astral UV is not installed.");
        return;
    }
    uvctrl::update().await.unwrap_or_else(|e| {
        log::error!("{}", e);
    });
}

/// Uninstall Astral UV
/// Uninstalls Astral UV from the system
///
/// # Arguments
/// * `input` - A reader for user input (e.g., stdin)
///
/// # Returns
/// * `Result<()>` - Ok if uninstalled
///
/// # Examples
/// ```
/// use pylot::uninstall;
/// use std::io;
///
/// uninstall(io::stdin());
/// ```
pub async fn uninstall<R: std::io::Read>(input: R) -> Result<()> {
    if (uvctrl::check("uv").await).is_err() {
        log::info!("Astral UV is not installed.");
        return Ok(());
    }
    uvctrl::uninstall(input).await.map_err(PylotError::Other)
}

/// List all available virtual environments
///
/// # Returns
/// * `()` - Nothing
///
/// # Examples
/// ```
/// use pylot::list;
///
/// list();
/// ```
pub async fn list() {
    venvmanager::VENVMANAGER.print_venv_table().await;
}

#[cfg(test)]
mod tests {
    use pylot_shared::logger;

    use super::*;

    #[tokio::test]
    async fn test_check() {
        logger::initialize_logger(log::LevelFilter::Trace);
        _ = check().await;
    }

    #[tokio::test]
    async fn test_list() {
        logger::initialize_logger(log::LevelFilter::Trace);
        list().await;
    }

    #[tokio::test]
    async fn test_delete() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let result = delete(io::stdin(), io::stdin(), Some("test_env")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_activate() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let result = activate(Some("test_env_not_here")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "cannot reliably test missing-UV scenario when CI pre-installs UV to a location not cleaned up by the uninstall script"]
    async fn test_create_missing_uv() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("y\n");
        let result_un = uninstall(cursor).await;
        assert!(result_un.is_ok());
        let result = create("test_env", Some("3.8"), None, None, false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_packages_from_requirements() {
        logger::initialize_logger(log::LevelFilter::Trace);
        use std::io::Write;

        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "pandas").unwrap();
        writeln!(temp_file, "scipy").unwrap();
        temp_file.flush().unwrap();

        let requirements = temp_file.path().to_str().unwrap();
        let mut packages = vec!["numpy".to_string()];

        let result = update_packages_from_requirements(requirements, &mut packages).await;
        assert!(result.is_ok());
        assert!(packages.contains(&"numpy".to_string()));
        assert!(packages.contains(&"pandas".to_string()));
        assert!(packages.contains(&"scipy".to_string()));

        // temp_file is automatically deleted when dropped
    }

    #[tokio::test]
    async fn test_create_missing_name() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let result = create("", None, None, None, false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_install_uv_no() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("n\n");
        let result_in = install(cursor.clone()).await;
        assert!(result_in.is_ok());
    }

    #[tokio::test]
    async fn test_uninstall_uv_no() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("n\n");
        let result_un = uninstall(cursor).await;
        assert!(result_un.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires network access to astral.sh CDN to download UV binary"]
    async fn test_install_uv_yes() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("y\n");
        let result_in = install(cursor.clone()).await;
        assert!(result_in.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires network access to astral.sh CDN to download UV binary"]
    async fn test_install_update_uv_yes() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("y\n");
        let result_in = install(cursor.clone()).await;
        update().await;
        assert!(result_in.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires network access to astral.sh CDN to download UV binary"]
    async fn test_uninstall_uv_yes() {
        logger::initialize_logger(log::LevelFilter::Trace);
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
    async fn test_uninstall_update_uv_yes() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("y\n");
        let result_un = uninstall(cursor).await;
        assert!(result_un.is_ok());
        update().await;
    }

    // ── update_packages_from_requirements – additional coverage ──────────────

    #[tokio::test]
    async fn test_update_packages_deduplication() {
        logger::initialize_logger(log::LevelFilter::Trace);
        use std::io::Write;

        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "numpy").unwrap(); // already in packages
        writeln!(temp_file, "scipy").unwrap();
        temp_file.flush().unwrap();

        let requirements = temp_file.path().to_str().unwrap();
        let mut packages = vec!["numpy".to_string()]; // pre-existing

        let result = update_packages_from_requirements(requirements, &mut packages).await;
        assert!(result.is_ok());
        // numpy must not be duplicated
        assert_eq!(packages.iter().filter(|p| p.as_str() == "numpy").count(), 1);
        assert!(packages.contains(&"scipy".to_string()));
    }

    #[tokio::test]
    async fn test_update_packages_nonexistent_file() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let mut packages = vec![];
        let result =
            update_packages_from_requirements("does_not_exist_req.txt", &mut packages).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_packages_empty_requirements() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let mut packages = vec!["existing".to_string()];
        // Passing an empty string should be a no-op (no file read attempted).
        let result = update_packages_from_requirements("", &mut packages).await;
        assert!(result.is_ok());
        assert_eq!(packages, vec!["existing".to_string()]);
    }

    // ── create – invalid package name ─────────────────────────────────────────

    #[tokio::test]
    async fn test_create_invalid_package_name() {
        logger::initialize_logger(log::LevelFilter::Trace);
        // A package containing a shell metacharacter should be rejected before
        // any network or FS operation occurs.
        let result = create(
            "valid_env",
            None,
            Some(vec!["bad;pkg".to_string()]),
            None,
            false,
        )
        .await;
        assert!(result.is_err());
    }

    // ── delete – venv not found by name ──────────────────────────────────────

    #[tokio::test]
    async fn test_delete_nonexistent_venv() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let result = delete(
            std::io::Cursor::new("y\n"),
            std::io::stdin(),
            Some("definitely_does_not_exist_venv"),
        )
        .await;
        assert!(result.is_err());
    }
}
