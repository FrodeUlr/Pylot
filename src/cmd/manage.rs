use crate::cmd::utils::{self, confirm};
use colored::Colorize;

pub async fn install() {
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

    if !confirm() {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd(cmd, args);
    utils::run_command(&mut child).await;
}

pub async fn uninstall() {
    println!("{}", "Uninstalling Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    //let cmd;
    //let args: &[&str];

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

    if !confirm() {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd(cmd, args);
    utils::run_command(&mut child).await;
}

pub async fn check() {
    println!(
        "{}",
        "Checking if Astral UV is installed and configured...".cyan()
    );

    let installed: bool = if cfg!(target_os = "windows") {
        utils::is_command_available("where", "uv").await
    } else {
        utils::is_command_available("which", "uv").await
    };

    if installed {
        println!("{}", "Astral UV was found".green());
        return;
    }

    println!(
        "{}",
        "Astral UV is not installed or missing from path".red()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check() {
        let installed = check().await;
        assert_eq!(installed, ());
    }
}
