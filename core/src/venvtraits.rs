use crate::error::Result;

/// Defines how a virtual environment is created.
pub trait Create {
    /// Create the virtual environment on disk.
    ///
    /// # Errors
    ///
    /// Returns [`PylotError`](crate::error::PylotError) if the environment
    /// already exists, the `uv` command fails, or an I/O error occurs.
    fn create(&self) -> impl std::future::Future<Output = Result<()>>;
}

/// Defines how a virtual environment is deleted.
pub trait Delete {
    /// Delete the virtual environment from disk.
    ///
    /// If `confirm` is `true`, the user is prompted via `input` before the
    /// directory is removed.
    ///
    /// # Errors
    ///
    /// Returns [`PylotError`](crate::error::PylotError) if the user cancels,
    /// the environment does not exist, or an I/O error occurs.
    fn delete<R: std::io::Read>(
        &self,
        input: R,
        confirm: bool,
    ) -> impl std::future::Future<Output = Result<()>>;
}

/// Defines how a virtual environment is activated.
pub trait Activate {
    /// Spawn a new shell with the virtual environment activated.
    ///
    /// On Unix this replaces the current process via `exec`; on Windows it
    /// spawns a child shell and blocks until it exits.
    ///
    /// # Errors
    ///
    /// Returns [`PylotError`](crate::error::PylotError) if the shell process
    /// cannot be spawned or the activation command fails.
    fn activate(&self) -> impl std::future::Future<Output = Result<()>>;
}
