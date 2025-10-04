use crate::{
    core::processes,
    utility::constants::{
        BASH_CMD, UV_UNIX_INSTALL_ARGS, UV_UNIX_UNINSTALL_ARGS, UV_WINGET_INSTALL_ARGS,
        UV_WINGET_UNINSTALL_ARGS, WINGET_CMD,
    },
    utils::confirm,
};
use colored::Colorize;

pub async fn install<R: std::io::Read>(input: R) -> Result<(), String> {
    println!("{}", "Installing Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
        (WINGET_CMD, UV_WINGET_INSTALL_ARGS)
    } else {
        (BASH_CMD, UV_UNIX_INSTALL_ARGS)
    };

    println!("{}", format!("  {} {}", cmd, args.join(" ")).red());

    if !confirm(input) {
        println!("{}", "Exiting...".yellow());
        return Ok(());
    }

    let mut child = processes::create_child_cmd(cmd, args, "");
    processes::run_command(&mut child)
        .await
        .map_err(|_| "Installation failed".to_string())?;
    Ok(())
}

pub async fn uninstall<R: std::io::Read>(input: R) -> Result<(), String> {
    println!("{}", "Uninstalling Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
        (WINGET_CMD, UV_WINGET_UNINSTALL_ARGS)
    } else {
        (BASH_CMD, UV_UNIX_UNINSTALL_ARGS)
    };

    println!("{}", format!("  {} {}", cmd, args.join(" ")).red());

    if !confirm(input) {
        println!("{}", "Exiting...".yellow());
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
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        #[cfg(unix)]
        {
            let cursor = std::io::Cursor::new("y\n");
            install(cursor).await.expect("Failed to install Astral UV");
        }
    }

    #[tokio::test]
    async fn test_install_uv_no() {
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        let cursor = std::io::Cursor::new("n\n");
        install(cursor).await.expect("Failed to install Astral UV");
    }

    #[tokio::test]
    async fn test_uninstall_uv_no() {
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        let cursor = std::io::Cursor::new("n\n");
        uninstall(cursor)
            .await
            .expect("Failed to uninstall Astral UV");
    }
}
