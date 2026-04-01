use thiserror::Error;

#[derive(Error, Debug)]
pub enum PylotError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    #[error("Virtual environment not found: {0}")]
    VenvNotFound(String),

    #[error("Virtual environment already exists: {0}")]
    VenvExists(String),

    #[error("Invalid virtual environment name: {0}")]
    InvalidVenvName(String),

    #[error("Invalid package name: {0}")]
    InvalidPackageName(String),

    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),

    #[error("Path error: {0}")]
    PathError(String),

    #[error("Settings error: {0}")]
    Settings(String),

    #[error("Cancelled by user")]
    Cancelled,

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
