// ── Windows-specific constants ────────────────────────────────────────────────

/// The `pwsh` (PowerShell 7+) executable name.
pub const PWSH_CMD: &str = "pwsh";
/// The legacy `powershell` (Windows PowerShell 5.x) executable name.
pub const POWERSHELL_CMD: &str = "powershell";
/// Relative path to the Python interpreter inside a Windows virtual environment.
pub const WIN_PYTHON_EXEC: &str = "Scripts/python.exe";
/// The Windows Package Manager CLI executable name.
pub const WINGET_CMD: &str = "winget";
/// `winget` arguments used to install Astral UV.
pub const UV_WINGET_INSTALL_ARGS: &[&str] = &["install", "astral-sh.uv"];
/// `winget` arguments used to upgrade Astral UV.
pub const UV_WINGET_UPGRADE_ARGS: &[&str] = &["upgrade", "astral-sh.uv"];
/// `winget` arguments used to uninstall Astral UV.
pub const UV_WINGET_UNINSTALL_ARGS: &[&str] = &["uninstall", "astral-sh.uv"];

// ── Unix-specific constants ───────────────────────────────────────────────────

/// The POSIX shell executable name.
pub const SH_CMD: &str = "sh";
/// Relative path to `python3` inside a Unix virtual environment.
pub const UNIX_PYTHON3_EXEC: &str = "bin/python3";
/// Relative path to `python` inside a Unix virtual environment.
pub const UNIX_PYTHON_EXEC: &str = "bin/python";
/// `sh -c` arguments that download and run the official UV install script.
pub const UV_UNIX_INSTALL_ARGS: &[&str] =
    &["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"];
/// `sh -c` arguments that remove the UV and UVX binaries from `~/.local/bin/`.
pub const UV_UNIX_UNINSTALL_ARGS: &[&str] = &["-c", "rm ~/.local/bin/uv ~/.local/bin/uvx"];

// ── Shared constants ──────────────────────────────────────────────────────────

/// The `uv` executable name.
pub const UV_COMMAND: &str = "uv";
/// `uv` arguments used to self-update on Unix.
pub const UPDATE_ARGS: &[&str] = &["self", "update"];
/// Default directory where Pylot stores managed virtual environments.
pub const DEFAULT_VENV_HOME: &str = "~/pylot/venvs/";
/// Default Python version used when none is specified.
pub const DEFAULT_PYTHON_VERSION: &str = "3.12";

// ── Error messages ────────────────────────────────────────────────────────────

/// Error message emitted when virtual environment creation fails.
pub const ERROR_CREATING_VENV: &str = "Error creating virtual environment";
/// Error message emitted when the requested virtual environment does not exist.
pub const ERROR_VENV_NOT_EXISTS: &str = "Virtual environment does not exist";

#[cfg(test)]
mod tests {
    use super::*;

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
