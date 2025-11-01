// Windows-specific constants
pub const PWSH_CMD: &str = "pwsh";
pub const POWERSHELL_CMD: &str = "powershell";
pub const WIN_PYTHON_EXEC: &str = "Scripts/python.exe";
pub const WINGET_CMD: &str = "winget";
pub const UV_WINGET_INSTALL_ARGS: &[&str] = &["install", "astral-sh.uv"];
pub const UV_WINGET_UNINSTALL_ARGS: &[&str] = &["uninstall", "astral-sh.uv"];

// Unix-specific constants
pub const BASH_CMD: &str = "bash";
pub const UNIX_PYTHON3_EXEC: &str = "bin/python3";
pub const UNIX_PYTHON_EXEC: &str = "bin/python";
pub const UV_UNIX_INSTALL_ARGS: &[&str] =
    &["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"];
pub const UV_UNIX_UNINSTALL_ARGS: &[&str] = &["-c", "rm ~/.local/bin/uv ~/.local/bin/uvx"];

// Shared constants
pub const UPDATE_COMMAND: &str = "uv";
pub const UPDATE_ARGS: &[&str] = &["self", "update"];

// Error messages
pub const ERROR_CREATING_VENV: &str = "Error creating virtual environment";
pub const ERROR_VENV_NOT_EXISTS: &str = "Virtual environment does not exist";

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
