use colored::Colorize;
use std::{
    io::{stdout, BufRead, Write},
    process::{Command as StdCommand, Stdio},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};

use crate::utility::constants::{POWERSHELL_CMD, PWSH_CMD};

pub fn create_child_cmd(cmd: &str, args: &[&str]) -> Child {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command")
}

pub fn create_child_cmd_run(cmd: &str, run: &str, args: &[&str]) -> Child {
    Command::new(cmd)
        .arg(run)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command")
}

pub fn activate_venv_shell(cmd: &str, args: Vec<String>) {
    let _ = StdCommand::new(cmd)
        .arg("-c")
        .args(args)
        .spawn()
        .expect("Failed to activate virtual environment")
        .wait();
}

pub async fn run_command(child: &mut Child) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{}", line.green());
        }
    });

    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("{}", line.yellow());
        }
    });

    let (stdout_res, stderr_res, child_res) = tokio::join!(stdout_task, stderr_task, child.wait());

    if let Err(e) = stdout_res {
        eprintln!("{}", format!("Error reading stdout: {}", e).red());
    };
    if let Err(e) = stderr_res {
        eprintln!("{}", format!("Error reading stderr: {}", e).red());
    };
    if let Err(e) = child_res {
        eprintln!("{}", format!("Error waiting for child: {}", e).red());
    }

    Ok(())
}

pub fn confirm<R: std::io::Read>(input: R) -> bool {
    let mut stdin = std::io::BufReader::new(input);
    print!("{}", "Do you want to continue? (y/n): ".cyan());
    let _ = stdout().flush();
    let mut input_string = String::new();
    if stdin.read_line(&mut input_string).is_ok() {
        matches!(input_string.trim(), "y" | "yes")
    } else {
        false
    }
}

pub fn get_parent_shell() -> String {
    if cfg!(target_os = "windows") {
        let shell = if which::which(PWSH_CMD).is_ok() {
            PWSH_CMD
        } else {
            POWERSHELL_CMD
        };
        return shell.to_string();
    }
    std::env::var("SHELL").unwrap()
}

pub fn exit_with_error(msg: &str) -> ! {
    eprintln!("{}", msg.red());
    std::process::exit(1);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_parent_shell() {
        let shell = get_parent_shell();
        if cfg!(target_os = "windows") {
            assert!(shell == "powershell" || shell == "pwsh");
        } else {
            assert!(!shell.is_empty());
        }
    }

    #[tokio::test]
    async fn test_create_child_cmd() {
        if cfg!(target_os = "windows") {
            let cmd = "cmd";
            let args = &["/C", "echo", "Hello"];
            let child = create_child_cmd(cmd, args);
            assert!(child.id() > Some(0));
        } else {
            let cmd = "ls";
            let args = &["-lah"];
            let child = create_child_cmd(cmd, args);
            assert!(child.id() > Some(0));
        }
    }

    #[tokio::test]
    async fn test_run_command() {
        if cfg!(target_os = "windows") {
            let cmd = "cmd";
            let args = &["/C", "echo", "Hello"];
            let mut child = create_child_cmd(cmd, args);
            let res = run_command(&mut child).await;
            assert!(res.is_ok());
        } else {
            let cmd = "ls";
            let args = &["-lah"];
            let mut child = create_child_cmd(cmd, args);
            let res = run_command(&mut child).await;
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_confirm_yes() {
        let cursor = std::io::Cursor::new("y\n");
        let result = confirm(cursor);
        assert!(result);
    }

    #[test]
    fn test_confirm_no() {
        let cursor = std::io::Cursor::new("n\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_invalid() {
        let cursor = std::io::Cursor::new("x\n");
        let result = confirm(cursor);
        assert!(!result);
    }

    #[test]
    fn test_confirm_empty() {
        let cursor = std::io::Cursor::new("\n");
        let result = confirm(cursor);
        assert!(!result);
    }
}
