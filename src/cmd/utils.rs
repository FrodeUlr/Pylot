use colored::Colorize;
use std::{io::stdin, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};

pub async fn is_command_available(cmd: &str, arg: &str) -> bool {
    Command::new(cmd)
        .arg(arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn create_child_cmd(cmd: &str, command: &str, args: &[&str]) -> Child {
    Command::new(cmd)
        .arg(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command")
}

pub async fn run_command(child: &mut Child) {
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

    let stderr_taskk = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{}", line.red());
        }
    });

    let (stdout_res, stderr_res, _) = tokio::join!(stdout_task, stderr_taskk, child.wait());

    if let Err(e) = stdout_res {
        eprintln!("{}", format!("Error reading stdout: {}", e).red());
    };
    if let Err(e) = stderr_res {
        eprintln!("{}", format!("Error reading stderr: {}", e).red());
    };

    let _ = child.wait().await;
}

pub fn confirm() -> bool {
    println!("{}", "Do you want to continue? (y/n): ".cyan());
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim() == "y"
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_is_command_available() {
        let available = is_command_available("ls", "--version").await;
        assert_eq!(available, true);
    }

    #[tokio::test]
    async fn test_create_child_cmd() {
        let cmd = "ls";
        let arg = "-lah";
        let args = &["|", "grep python-manager"];
        let child = create_child_cmd(cmd, arg, args);
        assert_eq!(child.id() > Some(0), true);
    }

    #[tokio::test]
    async fn test_run_command() {
        let cmd = "ls";
        let arg = "-lah";
        let args = &["|", "grep python-manager"];
        let mut child = create_child_cmd(cmd, arg, args);
        run_command(&mut child).await;
    }
}
