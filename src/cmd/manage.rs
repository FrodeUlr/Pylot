use crate::cmd::utils::{self, confirm};
use colored::Colorize;

pub async fn install<R: std::io::Read>(input: R) -> bool {
    println!("{}", "Installing Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let (cmd, args) = if cfg!(target_os = "windows") {
        let _cmd = "winget";
        let _args = &["install", "astral-sh.uv"];
        (_cmd, _args)
    } else {
        let _cmd = "bash";
        let args = &["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"];
        (_cmd, args)
    };

    println!("{}", format!("  {} {}", cmd, args.join(" ")).red());

    if !confirm(input) {
        println!("{}", "Exiting...".yellow());
        return false;
    }

    let mut child = utils::create_child_cmd(cmd, args);
    utils::run_command(&mut child).await;
    true
}

pub async fn uninstall<R: std::io::Read>(input: R) -> bool {
    println!("{}", "Uninstalling Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let (cmd, args) = if cfg!(target_os = "windows") {
        let _cmd = "winget";
        let _args = &["uninstall", "astral-sh.uv"];
        (_cmd, _args)
    } else {
        let _cmd = "bash";
        let _args = &["-c", "rm ~/.local/bin/uv ~/.local/bin/uvx"];
        (_cmd, _args)
    };

    println!("{}", format!("  {} {}", cmd, args.join(" ")).red());

    if !confirm(input) {
        println!("{}", "Exiting...".yellow());
        return false;
    }

    let mut child = utils::create_child_cmd(cmd, args);
    utils::run_command(&mut child).await;
    true
}

pub async fn check() -> bool {
    let installed: bool = if cfg!(target_os = "windows") {
        utils::is_command_available("where", "uv").await
    } else {
        utils::is_command_available("which", "uv").await
    };

    installed
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[tokio::test]
    async fn test_check() {
        let installed = check().await;
        assert_eq!(installed, false);
    }

    #[tokio::test]
    async fn test_install() {
        if cfg!(target_os = "windows") {
            // Github agent does not have winget, maybe add another install option
            assert_eq!(true, true);
            return;
        }
        let is_installed = check().await;
        let input = Cursor::new("y\n");
        let success = install(input).await;
        assert_eq!(success, true);
        if !is_installed {
            let input = Cursor::new("y\n");
            uninstall(input).await;
        }
        let end_status = check().await;
        assert_eq!(end_status, is_installed);
    }

    #[tokio::test]
    async fn test_uninstall() {
        if cfg!(target_os = "windows") {
            // Github agent does not have winget, maybe add another install option
            assert_eq!(true, true);
            return;
        }
        let is_installed = check().await;
        if !is_installed {
            let input = Cursor::new("y\n");
            install(input).await;
        }
        let input = Cursor::new("y\n");
        let success = uninstall(input).await;
        assert_eq!(success, true);
        if is_installed {
            let input = Cursor::new("y\n");
            install(input).await;
        }
        let end_status = check().await;
        assert_eq!(end_status, is_installed);
    }
}
