use crate::cmd::utils::{self, confirm};
use colored::Colorize;

pub async fn install() {
    println!("{}", "Installing Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let prog;
    let cmd;
    let args: &[&str];

    if cfg!(target_os = "windows") {
        prog = "winget";
        cmd = "install";
        args = &["astral-sh.uv"];
    } else {
        prog = "bash";
        cmd = "-c";
        args = &[
            "curl -LsSf https://astral.sh/uv/install.sh | sh",
        ];
    }

    println!("{}", format!("  {} {} {}", prog, cmd, args.join(" ")).red());

    if !confirm() {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd(prog, cmd, args);
    utils::run_command(&mut child).await;
}

pub async fn uninstall() {
    println!("{}", "Uninstalling Astral UV...".yellow());
    println!("{}", "This will run the following command:".yellow());

    let prog;
    let cmd;
    let arg: &[&str];

    if cfg!(target_os = "windows") {
        prog = "winget";
        cmd = "uninstall";
        arg = &["astral-sh.uv"];
    } else {
        prog = "bash";
        cmd = "-c";
        arg = &["rm ~/.local/bin/uv ~/.local/bin/uvx"];
    }

    println!("{}", format!("  {} {} {}", prog, cmd, arg.join(" ")).red());

    if !confirm() {
        println!("{}", "Exiting...".yellow());
        return;
    }

    let mut child = utils::create_child_cmd(prog, cmd, arg);
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
