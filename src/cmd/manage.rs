use crate::cmd::utils::{self, confirm};
use colored::Colorize;

pub async fn install<R: std::io::Read>(input: R) -> Result<(), String> {
    println!("{}", "Installing Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
        ("winget", &["install", "astral-sh.uv"])
    } else {
        (
            "bash",
            &["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"],
        )
    };

    println!("{}", format!("  {} {}", cmd, args.join(" ")).red());

    if !confirm(input) {
        println!("{}", "Exiting...".yellow());
        return Ok(());
    }

    let mut child = utils::create_child_cmd(cmd, args);
    utils::run_command(&mut child)
        .await
        .map_err(|_| "Installation failed".to_string())?;
    Ok(())
}

pub async fn uninstall<R: std::io::Read>(input: R) -> Result<(), String> {
    println!("{}", "Uninstalling Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
        ("winget", &["uninstall", "astral-sh.uv"])
    } else {
        ("bash", &["-c", "rm ~/.local/bin/uv ~/.local/bin/uvx"])
    };

    println!("{}", format!("  {} {}", cmd, args.join(" ")).red());

    if !confirm(input) {
        println!("{}", "Exiting...".yellow());
        return Ok(());
    }

    let mut child = utils::create_child_cmd(cmd, args);
    utils::run_command(&mut child)
        .await
        .map_err(|_| "Uninstallation failed".to_string())?;
    Ok(())
}

pub async fn check() -> bool {
    if cfg!(target_os = "windows") {
        utils::is_command_available("where", "uv").await
    } else {
        utils::is_command_available("which", "uv").await
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use tokio::time::{sleep, Duration};

    use super::*;

    #[tokio::test]
    async fn test_install() {
        if cfg!(target_os = "windows") {
            // Github agent does not have winget, maybe add another install option
            return;
        }
        let is_installed = check().await;
        let input = Cursor::new("y\n");
        let success = install(input).await;
        assert!(success.is_ok());
        if !is_installed {
            let input = Cursor::new("y\n");
            let res = uninstall(input).await;
            assert!(res.is_ok());
        }
        sleep(Duration::from_secs(2)).await;
        let end_status = check().await;
        assert_eq!(end_status, is_installed);
    }

    #[tokio::test]
    async fn test_uninstall() {
        //if cfg!(target_os = "windows") {
        //    // Github agent does not have winget, maybe add another install option
        //    return;
        //}
        let is_installed = check().await;
        if !is_installed {
            let input = Cursor::new("y\n");
            let res = install(input).await;
            assert!(res.is_ok());
        }
        let input = Cursor::new("y\n");
        let success = uninstall(input).await;
        assert!(success.is_ok());
        if is_installed {
            let input = Cursor::new("y\n");
            let res = install(input).await;
            assert!(res.is_ok());
        }
        let end_status = check().await;
        assert_eq!(end_status, is_installed);
    }
}
