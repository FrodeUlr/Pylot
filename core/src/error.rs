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
