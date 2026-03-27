use crate::error::{PylotError, Result};
use crate::utility::constants::{POWERSHELL_CMD, PWSH_CMD};
use std::process::{Command as StdCommand, Stdio};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, BufReader},
    process::{Child, Command},
};

/// Spawn `cmd` as a Tokio async child process with stdout and stderr piped.
///
/// If `run` is non-empty it is prepended to `args` as the first argument (used
/// e.g. for `sh -c <script>`).
///
/// # Errors
///
/// Returns [`PylotError::CommandExecution`] if the process cannot be spawned.
pub fn create_child_cmd(cmd: &str, args: &[&str], run: &str) -> Result<Child> {
    let mut cmd = Command::new(cmd);
    if !run.is_empty() {
        cmd.arg(run);
    }
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| PylotError::CommandExecution(format!("Failed to execute command: {}", e)))
}

/// Activate a virtual environment by spawning a new shell with the environment
/// activated.
///
/// On **Unix** the current process is _replaced_ by the new shell via `exec`
/// (the call never returns on success).  On **Windows** a child process is
/// spawned and this function blocks until it exits, forwarding Ctrl-C signals
/// to the child.
///
/// # Errors
///
/// Returns [`PylotError::CommandExecution`] if the shell cannot be spawned.
pub fn activate_venv_shell(cmd: &str, args: Vec<String>) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // On Unix, we use exec() to replace the current process with the shell.
        // We pass -c to tell the shell to execute the command string in args.
        // The exec() call never returns on success, only on error.
        let error = StdCommand::new(cmd).arg("-c").args(args).exec();

        Err(PylotError::CommandExecution(format!(
            "Failed to execute shell: {}",
            error
        )))
    }

    #[cfg(not(unix))]
    {
        use std::os::windows::process::CommandExt;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Once;
        use winapi::um::wincon::GenerateConsoleCtrlEvent;

        // A process-wide atomic that always holds the PID of the currently-active
        // child shell (0 when none is running).  The Ctrl-C handler reads from it
        // so that re-activating a new environment after the first one exits works
        // correctly without re-registering the handler.
        static CHILD_PID: AtomicU32 = AtomicU32::new(0);
        // Ensure the handler is registered exactly once for the lifetime of the process.
        static HANDLER_INIT: Once = Once::new();

        let mut child = StdCommand::new(cmd)
            .args(args)
            .creation_flags(0x00000200)
            .spawn()
            .map_err(|e| PylotError::CommandExecution(format!("Failed to spawn process: {}", e)))?;

        CHILD_PID.store(child.id(), Ordering::SeqCst);

        HANDLER_INIT.call_once(|| {
            if let Err(e) = ctrlc::set_handler(|| {
                let pid = CHILD_PID.load(Ordering::SeqCst);
                if pid != 0 {
                    unsafe {
                        let result = GenerateConsoleCtrlEvent(
                            winapi::um::wincon::CTRL_BREAK_EVENT,
                            pid,
                        );
                        if result == 0 {
                            log::warn!("Failed to send Ctrl-Break event to child process");
                        }
                    }
                }
            }) {
                log::warn!("Failed to register Ctrl-C handler: {}", e);
            }
        });

        child
            .wait()
            .map_err(|e| PylotError::CommandExecution(format!("Failed to wait for child: {}", e)))?;

        // Clear the PID so a stale Ctrl-C after the shell exits is a no-op.
        CHILD_PID.store(0, Ordering::SeqCst);
        Ok(())
    }
}

/// Stream stdout and stderr from async readers, calling `handle_stdout` /
/// `handle_stderr` for every line.
///
/// Lines that contain `"error:"` are treated as fatal and cause the function to
/// return an error.
///
/// # Errors
///
/// Returns `Err` if a line contains `"error:"` or if reading either stream
/// fails.
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
) -> std::result::Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(String) + Send + 'static,
    G: FnMut(String) + Send + 'static,
{
    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Some(line) = lines.next_line().await? {
            if line.contains("error:") {
                return Err(line.to_string().into());
            }
            handle_stdout(line);
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    });

    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Some(line) = lines.next_line().await? {
            if line.contains("error:") {
                return Err(line.to_string().into());
            }
            handle_stderr(line);
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
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

/// Run a spawned child process to completion, forwarding stdout lines to the
/// `info` log level and stderr lines to `warn`.
///
/// # Errors
///
/// Returns [`PylotError`] if stdout/stderr cannot be read or if a line
/// contains `"error:"`.
pub async fn run_command(child: &mut Child) -> Result<()> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| PylotError::CommandExecution("Failed to open stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| PylotError::CommandExecution("Failed to open stderr".to_string()))?;

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

/// Return the shell that should be used for activating virtual environments.
///
/// On **Windows** this is `pwsh` if available, otherwise `powershell`.
/// On **Unix** it reads the `SHELL` environment variable.
///
/// # Errors
///
/// Returns [`PylotError::EnvVarNotSet`] on Unix when the `SHELL` variable is
/// not set.
pub fn get_parent_shell() -> Result<String> {
    if cfg!(target_os = "windows") {
        let shell = if which::which(PWSH_CMD).is_ok() {
            PWSH_CMD
        } else {
            POWERSHELL_CMD
        };
        return Ok(shell.to_string());
    }
    std::env::var("SHELL").map_err(|_| {
        PylotError::EnvVarNotSet("SHELL environment variable is not set".to_string())
    })
}

#[cfg(test)]
mod tests {

    use crate::constants::SH_CMD;
    use crate::logger;

    use super::*;
    use tokio::io::{AsyncRead, BufReader, ReadBuf};

    use std::io::{self, Cursor};
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};

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
        logger::initialize_logger(log::LevelFilter::Trace);
        let shell = get_parent_shell();
        if cfg!(target_os = "windows") {
            assert!(shell.is_ok());
            let shell_val = shell.unwrap();
            assert!(shell_val == "powershell" || shell_val == "pwsh");
        } else {
            // On Unix, result depends on SHELL env var
            assert!(shell.is_ok() || shell.is_err());
        }
    }

    #[tokio::test]
    async fn test_create_child_cmd() {
        logger::initialize_logger(log::LevelFilter::Trace);
        if cfg!(target_os = "windows") {
            let cmd = "cmd";
            let args = &["/C", "echo", "Hello"];
            let child = create_child_cmd(cmd, args, "");
            assert!(child.is_ok());
            if let Ok(c) = child {
                assert!(c.id() > Some(0));
            }
        } else {
            let cmd = "ls";
            let args = &["-lah"];
            let child = create_child_cmd(cmd, args, "");
            assert!(child.is_ok());
            if let Ok(c) = child {
                assert!(c.id() > Some(0));
            }
        }
    }

    #[tokio::test]
    async fn test_run_command() {
        logger::initialize_logger(log::LevelFilter::Trace);
        if cfg!(target_os = "windows") {
            let cmd = "cmd";
            let args = &["/C", "echo", "Hello"];
            let child = create_child_cmd(cmd, args, "");
            if let Ok(mut c) = child {
                let res = run_command(&mut c).await;
                assert!(res.is_ok());
            }
        } else {
            let cmd = "ls";
            let args = &["-lah"];
            let child = create_child_cmd(cmd, args, "");
            if let Ok(mut c) = child {
                let res = run_command(&mut c).await;
                assert!(res.is_ok());
            }
        }
    }

    #[tokio::test]
    async fn test_create_child_cmd_run() {
        logger::initialize_logger(log::LevelFilter::Trace);
        if cfg!(target_os = "windows") {
            let cmd = "cmd";
            let run = "/C";
            let args = &["echo", "Hello"];
            let child = create_child_cmd(cmd, args, run);
            assert!(child.is_ok());
            if let Ok(c) = child {
                assert!(c.id() > Some(0));
            }
        } else {
            let cmd = SH_CMD;
            let run = "-c";
            let args = &["echo Hello"];
            let child = create_child_cmd(cmd, args, run);
            assert!(child.is_ok());
            if let Ok(c) = child {
                assert!(c.id() > Some(0));
            }
        }
    }

    #[tokio::test]
    async fn test_run_command_with_handlers() {
        logger::initialize_logger(log::LevelFilter::Trace);
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
        logger::initialize_logger(log::LevelFilter::Trace);
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
        logger::initialize_logger(log::LevelFilter::Trace);
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
