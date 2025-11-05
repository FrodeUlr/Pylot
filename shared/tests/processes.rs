#[cfg(test)]
mod tests {

    use shared::constants::SH_CMD;
    use shared::processes::{
        create_child_cmd, get_parent_shell, run_command, run_command_with_handlers,
    };
    use tokio::io::BufReader;

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
            let cmd = SH_CMD;
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
