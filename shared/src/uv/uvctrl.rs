use crate::{
    constants::{UPDATE_ARGS, UV_COMMAND, UV_WINGET_UPGRADE_ARGS},
    core::processes,
    utility::constants::{
        SH_CMD, UV_UNIX_INSTALL_ARGS, UV_UNIX_UNINSTALL_ARGS, UV_WINGET_INSTALL_ARGS,
        UV_WINGET_UNINSTALL_ARGS, WINGET_CMD,
    },
    utils::{self, confirm},
};

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

    let mut child = processes::create_child_cmd(cmd, args, "");
    processes::run_command(&mut child)
        .await
        .map_err(|_| "Installation failed".to_string())?;
    log::info!("Astral UV has been installed.");
    Ok(())
}

pub async fn update() -> Result<(), String> {
    log::info!("Updating Astral UV...");
    if cfg!(target_os = "windows") {
        utils::which_check(&[WINGET_CMD])
            .map_err(|e| format!("Winget is required for update: {}", e))?;
        let mut child = processes::create_child_cmd(WINGET_CMD, UV_WINGET_UPGRADE_ARGS, "");
        processes::run_command(&mut child)
            .await
            .map_err(|_| "Update failed".to_string())?;
    } else {
        utils::which_check(&[UV_COMMAND])
            .map_err(|e| format!("UV command is required for update: {}", e))?;
        let mut child = processes::create_child_cmd(UV_COMMAND, UPDATE_ARGS, "");
        processes::run_command(&mut child)
            .await
            .map_err(|_| "Update failed".to_string())?;
    }
    Ok(())
}

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

    let mut child = processes::create_child_cmd(cmd, args, "");
    processes::run_command(&mut child)
        .await
        .map_err(|_| "Uninstallation failed".to_string())?;
    log::info!("Astral UV has been uninstalled.");
    Ok(())
}

fn confirm_cmd<R: std::io::Read>(input: R, cmd: &str, args: &[&str]) -> Option<Result<(), String>> {
    log::info!("This will run the following command:\n");
    log::error!("\t{} {}\n", cmd, args.join(" "));
    if !confirm(input) {
        log::warn!("Exiting...");
        return Some(Ok(()));
    }
    None
}

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
    async fn test_check() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let is_installed = check("uv").await;
        if is_installed.is_ok() {
            assert!(is_installed.is_ok());
        } else {
            assert!(is_installed.is_err());
        }
    }

    #[tokio::test]
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
