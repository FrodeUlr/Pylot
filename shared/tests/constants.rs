#[cfg(test)]
mod tests {
    use shared::constants::{
        POWERSHELL_CMD, PWSH_CMD, SH_CMD, UNIX_PYTHON3_EXEC, UNIX_PYTHON_EXEC, UV_COMMAND,
        WINGET_CMD, WIN_PYTHON_EXEC,
    };

    #[test]
    fn test_constants() {
        assert_eq!(WINGET_CMD, "winget");
        assert_eq!(SH_CMD, "sh");
        assert_eq!(PWSH_CMD, "pwsh");
        assert_eq!(POWERSHELL_CMD, "powershell");
        assert_eq!(WIN_PYTHON_EXEC, "Scripts/python.exe");
        assert_eq!(UNIX_PYTHON3_EXEC, "bin/python3");
        assert_eq!(UNIX_PYTHON_EXEC, "bin/python");
        assert_eq!(UV_COMMAND, "uv");
    }
}
