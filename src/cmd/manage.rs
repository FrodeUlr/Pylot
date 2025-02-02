use colored::Colorize;
use crate::cmd::utils::{self, confirm};

pub async fn install() {
    println!("{}", "Installing Astral UV...".cyan());
    println!("{}", "This will run the following command:".yellow());
    if cfg!(target_os = "windows") {
        install_windows().await;
        return;
    }

    install_linux().await;
}

async fn install_linux() {
    println!("{}", "  curl -LsSf https://astral.sh/uv/install.sh | sh".red());

    if confirm() == false {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd("bash", "-c", "curl -LsSf https://astral.sh/uv/install.sh | sh");

    utils::run_command(&mut child).await;
}

async fn install_windows() {
    println!("{}", "  winget install astral-sh.uv".red());

    if confirm() == false {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd("winget", "install", "astral-sh.uv");

    utils::run_command(&mut child).await;
}

pub async fn uninstall() {
    println!("{}", "Uninstalling Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());
    if cfg!(target_os = "windows") {
        uninstall_windows().await;
        return;
    }
    uninstall_linux().await;
}

async fn uninstall_linux() {
    println!("{}", "  rm ~/.local/bin/uv ~/.local/bin/uvx".red());
    if confirm() == false {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd("bash", "-c", "rm ~/.local/bin/uv ~/.local/bin/uvx");

    utils::run_command(&mut child).await;
}

async fn uninstall_windows() {
    println!("{}", "  winget uninstall astral-sh.uv".red());
    if confirm() == false {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd("winget", "uninstall", "astral-sh.uv");

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
