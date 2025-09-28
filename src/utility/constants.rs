pub const WINGET_CMD: &str = "winget";
pub const BASH_CMD: &str = "bash";
pub const PWSH_CMD: &str = "pwsh";
pub const POWERSHELL_CMD: &str = "powershell";
pub const WIN_PYTHON_EXEC: &str = "Scripts/python.exe";
pub const UNIX_PYTHON3_EXEC: &str = "bin/python3";
pub const UNIX_PYTHON_EXEC: &str = "bin/python";

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(WINGET_CMD, "winget");
        assert_eq!(BASH_CMD, "bash");
        assert_eq!(PWSH_CMD, "pwsh");
        assert_eq!(POWERSHELL_CMD, "powershell");
        assert_eq!(WIN_PYTHON_EXEC, "Scripts/python.exe");
        assert_eq!(UNIX_PYTHON3_EXEC, "bin/python3");
        assert_eq!(UNIX_PYTHON_EXEC, "bin/python");
    }
}
