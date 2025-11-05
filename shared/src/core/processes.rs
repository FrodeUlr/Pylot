use crate::utility::constants::{POWERSHELL_CMD, PWSH_CMD};
use std::process::{Command as StdCommand, Stdio};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, BufReader},
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

pub async fn run_command_with_handlers<
    RO: AsyncBufRead + Unpin + Send + 'static,
    RE: AsyncBufRead + Unpin + Send + 'static,
    F,
    G,
>(
    stdout_reader: RO,
    stderr_reader: RE,
    mut handle_stdout: F,
    mut handle_stderr: G,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(String) + Send + 'static,
    G: FnMut(String) + Send + 'static,
{
    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Some(line) = lines.next_line().await? {
            handle_stdout(line);
        }
        Ok::<(), Box<std::io::Error>>(())
    });

    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Some(line) = lines.next_line().await? {
            handle_stderr(line);
        }
        Ok::<(), Box<std::io::Error>>(())
    });

    let (stdout_res, stderr_res) = tokio::join!(stdout_task, stderr_task);

    if let Err(e) = stdout_res {
        return Err(format!("Error reading stdout: {}", e).into());
    }
    if let Err(e) = stderr_res {
        return Err(format!("Error reading stderr: {}", e).into());
    }

    if let Some(e) = stdout_res.ok().and_then(|r| r.err()) {
        return Err(format!("Error in stdout task: {}", e).into());
    }
    if let Some(e) = stderr_res.ok().and_then(|r| r.err()) {
        return Err(format!("Error in stderr task: {}", e).into());
    }

    Ok(())
}

pub async fn run_command(child: &mut Child) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    run_command_with_handlers(
        stdout_reader,
        stderr_reader,
        |line| log::info!("{}", line),
        |line| log::warn!("{}", line),
    )
    .await?;
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
