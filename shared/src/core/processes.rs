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

async fn run_command_with_handlers<
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

#[cfg(test)]
mod tests {

    use super::*;

    use std::io::Cursor;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};
    use tokio::io::{self, AsyncBufRead, AsyncRead, ReadBuf};

    struct ErrorReader;

    impl AsyncRead for ErrorReader {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::other("simulated error".to_string())))
        }
    }

    impl AsyncBufRead for ErrorReader {
        fn poll_fill_buf(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
            Poll::Ready(Err(io::Error::other("simulated error".to_string())))
        }
        fn consume(self: Pin<&mut Self>, _amt: usize) {}
    }

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

    #[tokio::test]
    async fn test_run_command_with_handlers() {
        let stdout_data = Cursor::new("line1\nline2\n");
        let stderr_data = Cursor::new("err1\nerr2\n");

        let stdout_lines = Arc::new(Mutex::new(Vec::new()));
        let stderr_lines = Arc::new(Mutex::new(Vec::new()));

        let stdout_lines_clone = Arc::clone(&stdout_lines);
        let stderr_lines_clone = Arc::clone(&stderr_lines);

        run_command_with_handlers(
            BufReader::new(stdout_data),
            BufReader::new(stderr_data),
            move |line| stdout_lines_clone.lock().unwrap().push(line),
            move |line| stderr_lines_clone.lock().unwrap().push(line),
        )
        .await
        .unwrap();

        assert_eq!(*stdout_lines.lock().unwrap(), vec!["line1", "line2"]);
        assert_eq!(*stderr_lines.lock().unwrap(), vec!["err1", "err2"]);
    }

    #[tokio::test]
    async fn test_run_command_with_handlers_stdout_err() {
        let stderr_data = Cursor::new("err1\nerr2\n");

        let stdout_lines = Arc::new(Mutex::new(Vec::new()));
        let stderr_lines = Arc::new(Mutex::new(Vec::new()));

        let stdout_lines_clone = Arc::clone(&stdout_lines);
        let stderr_lines_clone = Arc::clone(&stderr_lines);

        let result = run_command_with_handlers(
            ErrorReader,
            BufReader::new(stderr_data),
            move |line| stdout_lines_clone.lock().unwrap().push(line),
            move |line| stderr_lines_clone.lock().unwrap().push(line),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_command_with_handlers_stderr_err() {
        let stdout_data = Cursor::new("out\nout\n");

        let stdout_lines = Arc::new(Mutex::new(Vec::new()));
        let stderr_lines = Arc::new(Mutex::new(Vec::new()));

        let stdout_lines_clone = Arc::clone(&stdout_lines);
        let stderr_lines_clone = Arc::clone(&stderr_lines);

        let result = run_command_with_handlers(
            BufReader::new(stdout_data),
            ErrorReader,
            move |line| stdout_lines_clone.lock().unwrap().push(line),
            move |line| stderr_lines_clone.lock().unwrap().push(line),
        )
        .await;

        assert!(result.is_err());
    }
}
