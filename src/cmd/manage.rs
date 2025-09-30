use crate::cmd::utils::{self, confirm};
use crate::utility::constants::{BASH_CMD, WINGET_CMD};
use colored::Colorize;

pub async fn install<R: std::io::Read>(input: R) -> Result<(), String> {
    println!("{}", "Installing Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
        (WINGET_CMD, &["install", "astral-sh.uv"])
    } else {
        (
            BASH_CMD,
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
        (WINGET_CMD, &["uninstall", "astral-sh.uv"])
    } else {
        (BASH_CMD, &["-c", "rm ~/.local/bin/uv ~/.local/bin/uvx"])
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
}
