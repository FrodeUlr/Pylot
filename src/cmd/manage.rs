use colored::Colorize;
use crate::cmd::utils::{self, confirm};

pub async fn install() {
    println!("{}", "Installing Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let cmd;
    let arg: &[&str];
    let command;

    if cfg!(target_os = "windows") {
        cmd = "winget";
        command = "install";
        arg = &["astral-sh.uv"];
    } else {
        cmd = "bash";
        command = "-c";
        arg = &["curl", "-LsSf", "https://astral.sh/uv/install.sh", "|", "sh"];
    }

    println!("{}", format!("  {} {}", cmd, arg.join(" ")).red());

    if confirm() == false {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd(cmd, command, arg);
    utils::run_command(&mut child).await;
}

pub async fn uninstall() {
    println!("{}", "Uninstalling Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let cmd;
    let arg: &[&str];
    let command;

    if cfg!(target_os = "windows") {
        cmd = "winget";
        command = "uninstall";
        arg = &["astral-sh.uv"];
    } else {
        cmd = "bash";
        command = "-c";
        arg = &["rm", "~/.local/bin/uv", "~/.local/bin/uvx"];
    }

    println!("{}", format!("  {} {} {}", cmd, command, arg.join(" ")).red());

    if confirm() == false {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd(cmd, command, arg);
    utils::run_command(&mut child).await;
}

pub async fn check() {
    println!("{}", "Checking if Astral UV is installed and configured...".cyan());

    let installed: bool;

    if cfg!(target_os = "windows") {
        installed = utils::is_command_available("where", "uv").await;
    } else {
        installed = utils::is_command_available("which", "uv").await;
    }

    if installed {
        println!("{}", "Astral UV was found".green());
        return;
    }

    println!("{}", "Astral UV is not installed or missing from path".red());
}
