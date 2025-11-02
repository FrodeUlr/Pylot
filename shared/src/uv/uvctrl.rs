use crate::{
    core::processes,
    utility::constants::{
        BASH_CMD, UV_UNIX_INSTALL_ARGS, UV_UNIX_UNINSTALL_ARGS, UV_WINGET_INSTALL_ARGS,
        UV_WINGET_UNINSTALL_ARGS, WINGET_CMD,
    },
    utils::confirm,
};

pub async fn install<R: std::io::Read>(input: R) -> Result<(), String> {
    log::info!("{}", "Installing Astral UV...");
    log::info!("{}", "This will run the following command:");

    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
        (WINGET_CMD, UV_WINGET_INSTALL_ARGS)
    } else {
        (BASH_CMD, UV_UNIX_INSTALL_ARGS)
    };

    log::error!("\t{} {}", cmd, args.join(" "));

    if !confirm(input) {
        log::info!("{}", "Exiting...");
        return Ok(());
    }

    let mut child = processes::create_child_cmd(cmd, args, "");
    processes::run_command(&mut child)
        .await
        .map_err(|_| "Installation failed".to_string())?;
    Ok(())
}

pub async fn update() -> Result<(), String> {
    log::info!("{}", "Updating Astral UV...");
    #[cfg(unix)]
    {
        use crate::constants::{UPDATE_ARGS, UPDATE_COMMAND};
        let mut child = processes::create_child_cmd(UPDATE_COMMAND, UPDATE_ARGS, "");
        processes::run_command(&mut child)
            .await
            .map_err(|_| "Update failed".to_string())?;
    }
    #[cfg(not(unix))]
    {
        use crate::constants::UV_WINGET_UPGRADE_ARGS;

        let mut child = processes::create_child_cmd(WINGET_CMD, UV_WINGET_UPGRADE_ARGS, "");
        processes::run_command(&mut child)
            .await
            .map_err(|_| "Update failed".to_string())?;
    }
    Ok(())
}

pub async fn uninstall<R: std::io::Read>(input: R) -> Result<(), String> {
    log::info!("{}", "Uninstalling Astral UV...");
    log::info!("{}", "This will run the following command:");

    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
        (WINGET_CMD, UV_WINGET_UNINSTALL_ARGS)
    } else {
        (BASH_CMD, UV_UNIX_UNINSTALL_ARGS)
    };

    log::error!("\t{} {}", cmd, args.join(" "));

    if !confirm(input) {
        log::info!("{}", "Exiting...");
        return Ok(());
    }

    let mut child = processes::create_child_cmd(cmd, args, "");
    processes::run_command(&mut child)
        .await
        .map_err(|_| "Uninstallation failed".to_string())?;
    Ok(())
}

pub async fn check() -> bool {
    which::which("uv").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check() {
        let is_installed = check().await;
        if is_installed {
            println!("Astral UV is installed.");
            assert!(is_installed);
        } else {
            println!("Astral UV is not installed.");
            assert!(!is_installed);
        }
    }

    #[tokio::test]
    async fn test_install_uv_yes() {
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            install(cursor).await.expect("Failed to install Astral UV");
        }
    }

    #[tokio::test]
    async fn test_install_uv_no() {
        let cursor = std::io::Cursor::new("n\n");
        install(cursor).await.expect("Failed to install Astral UV");
    }

    #[tokio::test]
    async fn test_uninstall_uv_no() {
        let cursor = std::io::Cursor::new("n\n");
        uninstall(cursor)
            .await
            .expect("Failed to uninstall Astral UV");
    }
}
