use colored::Colorize;
use std::io::stdin;
use crate::cmd::utils;

pub async fn install_uv() {
    println!("{}", "Installing Astral UV...".cyan());
    println!("{}", "This will run the following command:".yellow());
    if cfg!(target_os = "windows") {
        install_uv_windows().await;
        return;
    }

    install_uv_linux().await;
}

async fn install_uv_linux() {
    println!("{}", "  curl -LsSf https://astral.sh/uv/install.sh | sh".red());

    if confirm() == false {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd("bash", "-c", "curl -LsSf https://astral.sh/uv/install.sh | sh");

    utils::run_command(&mut child).await;
}

async fn install_uv_windows() {
    println!("{}", "  winget install astral-sh.uv".red());

    if confirm() == false {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd("winget", "install", "astral-sh.uv");

    utils::run_command(&mut child).await;
}

fn confirm() -> bool {
    println!("{}", "Do you want to continue? (y/n): ".cyan());
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim() == "y"
}

pub async fn check_uv() {
    println!("{}", "Checking if Astral UV is installed...".cyan());
    let installed: bool;
    if cfg!(target_os = "windows") {
        installed = utils::is_command_available("where", "uv").await;
    } else {
        installed = utils::is_command_available("which", "uv").await;
    }
    if installed {
        println!("{}", "Astral UV is installed".green());
        return;
    }
    println!("{}", "Astral UV is not installed".red());
}
