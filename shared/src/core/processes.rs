use crate::utility::constants::{POWERSHELL_CMD, PWSH_CMD};
use colored::Colorize;
use std::process::{Command as StdCommand, Stdio};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};

pub fn create_child_cmd(cmd: &str, args: &[&str], run: &str) -> Child {
    let mut cmd = Command::new(cmd);
    if !run.is_empty() {
        cmd.arg(run);
    }
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command")
}

pub fn activate_venv_shell(cmd: &str, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = StdCommand::new(cmd).arg("-c").args(args).exec();

        Err(format!("Failed to execute shell: {}", error).into())
    }

    #[cfg(not(unix))]
    {
        use ctrlc;
        use std::os::windows::process::CommandExt;
        use std::sync::{Arc, Mutex};
        use winapi::um::wincon::GenerateConsoleCtrlEvent;

        let mut child = StdCommand::new(cmd)
            .arg("-c")
            .args(args)
            .creation_flags(0x00000200)
            .spawn()?;

        let child_id = child.id();
        let running = Arc::new(Mutex::new(true));
        let running_clone = running.clone();

        ctrlc::set_handler(move || {
            unsafe {
                GenerateConsoleCtrlEvent(winapi::um::wincon::CTRL_BREAK_EVENT, child_id);
            }
            let mut r = running_clone.lock().unwrap();
            *r = false;
        })
        .expect("Error setting Ctrl-C handler");

        child.wait()?;
        Ok(())
    }
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
        return Err(format!("Error waiting for child: {}", e).into());
    }
    let status = child_res.unwrap();
    if !status.success() {
        return Err(format!("Command exited with status: {}", status).into());
    }

    Ok(())
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
            let child = create_child_cmd(cmd, args, "");
            assert!(child.id() > Some(0));
        } else {
            let cmd = "ls";
            let args = &["-lah"];
            let child = create_child_cmd(cmd, args, "");
            assert!(child.id() > Some(0));
        }
    }

    #[tokio::test]
    async fn test_run_command() {
        if cfg!(target_os = "windows") {
            let cmd = "cmd";
            let args = &["/C", "echo", "Hello"];
            let mut child = create_child_cmd(cmd, args, "");
            let res = run_command(&mut child).await;
            assert!(res.is_ok());
        } else {
            let cmd = "ls";
            let args = &["-lah"];
            let mut child = create_child_cmd(cmd, args, "");
            let res = run_command(&mut child).await;
            assert!(res.is_ok());
        }
    }

    #[tokio::test]
    async fn test_create_child_cmd_run() {
        if cfg!(target_os = "windows") {
            let cmd = "cmd";
            let run = "/C";
            let args = &["echo", "Hello"];
            let child = create_child_cmd(cmd, args, run);
            assert!(child.id() > Some(0));
        } else {
            let cmd = "sh";
            let run = "-c";
            let args = &["echo Hello"];
            let child = create_child_cmd(cmd, args, run);
            assert!(child.id() > Some(0));
        }
    }
}
