use crate::{
    constants::{UPDATE_ARGS, UV_COMMAND, UV_WINGET_UPGRADE_ARGS},
    infra::processes,
    utility::constants::{
        SH_CMD, UV_UNIX_INSTALL_ARGS, UV_UNIX_UNINSTALL_ARGS, UV_WINGET_INSTALL_ARGS,
        UV_WINGET_UNINSTALL_ARGS, WINGET_CMD,
    },
    utils::{self, confirm},
};

/// Install Astral UV.
///
/// On **Windows** this delegates to `winget install astral-sh.uv`.
/// On **Unix** it downloads and runs the official installation shell script via
/// `curl … | sh`.
///
/// The user is shown the exact command that will run and prompted to confirm
/// before anything is executed.  Passing `"n\n"` (or any non-`y` response) as
/// `input` cancels the operation without error.
///
/// # Errors
///
/// Returns `Err(String)` if a required prerequisite (`winget`, `curl`, …) is
/// missing or if the underlying command fails.
pub async fn install<R: std::io::Read>(input: R) -> Result<(), String> {
    log::info!("Installing Astral UV...");

    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
        utils::which_check(&[WINGET_CMD])
            .map_err(|e| format!("Winget is required for installation(https://learn.microsoft.com/en-us/windows/package-manager/winget/): {}", e))?;
        (WINGET_CMD, UV_WINGET_INSTALL_ARGS)
    } else {
        utils::which_check(&[SH_CMD, "curl", "sh"]).map_err(|e| format!("{}", e))?;
        (SH_CMD, UV_UNIX_INSTALL_ARGS)
    };

    if let Some(value) = confirm_cmd(input, cmd, args) {
        return value;
    }

    let mut child = processes::create_child_cmd(cmd, args, "")
        .map_err(|e| format!("Failed to create command: {}", e))?;
    processes::run_command(&mut child)
        .await
        .map_err(|e| format!("Installation failed: {}", e))?;
    log::info!("Astral UV has been installed.");
    Ok(())
}

/// Update Astral UV to the latest version.
///
/// On **Windows** this runs `winget upgrade astral-sh.uv`.
/// On **Unix** it runs `uv self update`.
///
/// # Errors
///
/// Returns `Err(String)` if `winget` / `uv` is not found on `PATH` or if the
/// underlying update command fails.
pub async fn update() -> Result<(), String> {
    log::info!("Updating Astral UV...");
    if cfg!(target_os = "windows") {
        utils::which_check(&[WINGET_CMD])
            .map_err(|e| format!("Winget is required for update: {}", e))?;
        let mut child = processes::create_child_cmd(WINGET_CMD, UV_WINGET_UPGRADE_ARGS, "")
            .map_err(|e| format!("Failed to create command: {}", e))?;
        processes::run_command(&mut child)
            .await
            .map_err(|e| format!("Update failed: {}", e))?;
    } else {
        utils::which_check(&[UV_COMMAND])
            .map_err(|e| format!("UV command is required for update: {}", e))?;
        let mut child = processes::create_child_cmd(UV_COMMAND, UPDATE_ARGS, "")
            .map_err(|e| format!("Failed to create command: {}", e))?;
        processes::run_command(&mut child)
            .await
            .map_err(|e| format!("Update failed: {}", e))?;
    }
    Ok(())
}

/// Uninstall Astral UV.
///
/// On **Windows** this delegates to `winget uninstall astral-sh.uv`.
/// On **Unix** it removes the `uv` and `uvx` binaries from
/// `~/.local/bin/`.
///
/// The user is shown the command that will run and prompted to confirm before
/// anything is executed.
///
/// # Errors
///
/// Returns `Err(String)` if `winget` is not available (Windows) or if the
/// underlying command fails.
pub async fn uninstall<R: std::io::Read>(input: R) -> Result<(), String> {
    log::info!("Uninstalling Astral UV...");
    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
        utils::which_check(&[WINGET_CMD])
            .map_err(|e| format!("Winget is required for installation(https://learn.microsoft.com/en-us/windows/package-manager/winget/): {}", e))?;
        (WINGET_CMD, UV_WINGET_UNINSTALL_ARGS)
    } else {
        (SH_CMD, UV_UNIX_UNINSTALL_ARGS)
    };

    if let Some(value) = confirm_cmd(input, cmd, args) {
        return value;
    }

    let mut child = processes::create_child_cmd(cmd, args, "")
        .map_err(|e| format!("Failed to create command: {}", e))?;
    processes::run_command(&mut child)
        .await
        .map_err(|e| format!("Uninstallation failed: {}", e))?;
    log::info!("Astral UV has been uninstalled.");
    Ok(())
}

fn confirm_cmd<R: std::io::Read>(input: R, cmd: &str, args: &[&str]) -> Option<Result<(), String>> {
    log::info!("This will run the following command:\n");
    log::info!("\t{} {}\n", cmd, args.join(" "));
    if !confirm(input) {
        log::warn!("Exiting...");
        return Some(Ok(()));
    }
    None
}

/// Check whether the binary named `name` is present on `PATH`.
///
/// Returns `Ok(message)` when the binary is found, or `Err(message)` when it
/// is not.
pub async fn check(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    match which::which(name)
        .map(|_| ())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    {
        Ok(_) => Ok(format!("{} is installed.", name)),
        Err(e) => Err(format!("{} not found: {}", name, e).into()),
    }
}

#[cfg(test)]
mod tests {
    use crate::logger;

    use super::*;

    #[tokio::test]
    async fn test_check_command_exists() {
        logger::initialize_logger(log::LevelFilter::Trace);
        // Test with a command that should exist on all systems
        let result = check("sh").await;
        // On Unix systems, sh should exist
        #[cfg(unix)]
        assert!(result.is_ok());
        
        // On Windows, sh might not exist
        #[cfg(not(unix))]
        let _ = result; // Just verify it doesn't panic
    }

    #[tokio::test]
    #[ignore = "requires network access to astral.sh CDN to download UV binary"]
    async fn test_install_uv_yes() {
        logger::initialize_logger(log::LevelFilter::Trace);
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            install(cursor).await.expect("Failed to install Astral UV");
        }
    }

    #[tokio::test]
    async fn test_install_uv_no() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("n\n");
        install(cursor).await.expect("Failed to install Astral UV");
    }

    #[tokio::test]
    async fn test_uninstall_uv_no() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let cursor = std::io::Cursor::new("n\n");
        uninstall(cursor)
            .await
            .expect("Failed to uninstall Astral UV");
    }

    #[tokio::test]
    async fn test_update_uv() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let result = update().await;
        match result {
            Ok(_) => println!("Astral UV updated successfully."),
            Err(e) => println!("Failed to update Astral UV: {}", e),
        }
    }

    #[tokio::test]
    #[ignore = "requires network access to astral.sh CDN to download UV binary"]
    async fn test_install_uv_yes_update() {
        logger::initialize_logger(log::LevelFilter::Trace);
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            install(cursor).await.expect("Failed to install Astral UV");
            let result = update().await;
            match result {
                Ok(_) => println!("Astral UV updated successfully."),
                Err(e) => println!("Failed to update Astral UV: {}", e),
            }
        }
    }
}
