use thiserror::Error;

/// The unified error type used throughout the Pylot workspace.
///
/// Every public API returns [`Result<T>`] which is an alias for
/// `std::result::Result<T, PylotError>`.
#[derive(Error, Debug)]
pub enum PylotError {
    /// Wraps a standard [`std::io::Error`].
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// A subprocess or shell command returned a non-zero exit code or could not
    /// be spawned.
    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    /// The requested virtual environment directory does not exist.
    #[error("Virtual environment not found: {0}")]
    VenvNotFound(String),

    /// A virtual environment with the same name already exists.
    #[error("Virtual environment already exists: {0}")]
    VenvExists(String),

    /// The supplied virtual environment name contains illegal characters or is
    /// otherwise rejected by the validation rules.
    #[error("Invalid virtual environment name: {0}")]
    InvalidVenvName(String),

    /// A package name failed validation (e.g. contains shell metacharacters).
    #[error("Invalid package name: {0}")]
    InvalidPackageName(String),

    /// A required environment variable (e.g. `HOME`) is not set.
    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),

    /// A filesystem path operation failed for a reason not covered by
    /// [`PylotError::Io`].
    #[error("Path error: {0}")]
    PathError(String),

    /// The `settings.toml` file could not be read or deserialized.
    #[error("Settings error: {0}")]
    Settings(String),

    /// The user explicitly cancelled an interactive prompt.
    #[error("Cancelled by user")]
    Cancelled,

    /// A catch-all variant for errors that do not fit any of the more specific
    /// variants above.
    #[error("{0}")]
    Other(String),
}

impl From<String> for PylotError {
    fn from(s: String) -> Self {
        PylotError::Other(s)
    }
}

impl From<&str> for PylotError {
    fn from(s: &str) -> Self {
        PylotError::Other(s.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for PylotError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        PylotError::Other(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PylotError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::other("disk full");
        let err = PylotError::Io(io_err);
        assert!(err.to_string().contains("IO error:"));
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn test_command_execution_display() {
        let err = PylotError::CommandExecution("exit code 1".to_string());
        assert_eq!(err.to_string(), "Command execution failed: exit code 1");
    }

    #[test]
    fn test_venv_not_found_display() {
        let err = PylotError::VenvNotFound("myenv".to_string());
        assert_eq!(err.to_string(), "Virtual environment not found: myenv");
    }

    #[test]
    fn test_venv_exists_display() {
        let err = PylotError::VenvExists("myenv".to_string());
        assert_eq!(err.to_string(), "Virtual environment already exists: myenv");
    }

    #[test]
    fn test_invalid_venv_name_display() {
        let err = PylotError::InvalidVenvName("bad/name".to_string());
        assert_eq!(
            err.to_string(),
            "Invalid virtual environment name: bad/name"
        );
    }

    #[test]
    fn test_invalid_package_name_display() {
        let err = PylotError::InvalidPackageName("bad;pkg".to_string());
        assert_eq!(err.to_string(), "Invalid package name: bad;pkg");
    }

    #[test]
    fn test_env_var_not_set_display() {
        let err = PylotError::EnvVarNotSet("SHELL".to_string());
        assert_eq!(err.to_string(), "Environment variable not set: SHELL");
    }

    #[test]
    fn test_path_error_display() {
        let err = PylotError::PathError("no such path".to_string());
        assert_eq!(err.to_string(), "Path error: no such path");
    }

    #[test]
    fn test_settings_error_display() {
        let err = PylotError::Settings("missing key".to_string());
        assert_eq!(err.to_string(), "Settings error: missing key");
    }

    #[test]
    fn test_cancelled_display() {
        let err = PylotError::Cancelled;
        assert_eq!(err.to_string(), "Cancelled by user");
    }

    #[test]
    fn test_other_display() {
        let err = PylotError::Other("something went wrong".to_string());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_from_string() {
        let err: PylotError = "from str literal".to_string().into();
        assert!(matches!(err, PylotError::Other(_)));
        assert_eq!(err.to_string(), "from str literal");
    }

    #[test]
    fn test_from_str_slice() {
        let err: PylotError = "from &str".into();
        assert!(matches!(err, PylotError::Other(_)));
        assert_eq!(err.to_string(), "from &str");
    }

    #[test]
    fn test_from_box_dyn_error() {
        let boxed: Box<dyn std::error::Error> = "boxed error".into();
        let err: PylotError = boxed.into();
        assert!(matches!(err, PylotError::Other(_)));
        assert_eq!(err.to_string(), "boxed error");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::other("io failure");
        let err: PylotError = io_err.into();
        assert!(matches!(err, PylotError::Io(_)));
    }

    #[test]
    fn test_result_ok() {
        let r: i32 = 42;
        assert_eq!(r, 42);
    }

    #[test]
    fn test_result_err() {
        let r: Result<i32> = Err(PylotError::Cancelled);
        assert!(r.is_err());
    }

    #[test]
    fn test_debug_format() {
        let err = PylotError::VenvNotFound("env1".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("VenvNotFound"));
    }
}
