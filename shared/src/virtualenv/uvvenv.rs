use crate::error::{PylotError, Result};
use crate::{
    constants::{DEFAULT_VENV_HOME, ERROR_VENV_NOT_EXISTS, POWERSHELL_CMD, PWSH_CMD, SH_CMD},
    processes, settings, utils, uvctrl,
    venvtraits::{Activate, Create, Delete},
};
use colored::Colorize;
use std::borrow::Cow;
use tokio::fs as async_fs;

/// A Python virtual environment managed by Astral UV.
///
/// `UvVenv` is the concrete implementation of the [`Create`], [`Delete`], and
/// [`Activate`] traits.  Instances are typically obtained via
/// [`VenvManager::list`](crate::virtualenv::venvmanager::VenvManager::list) or
/// [`VenvManager::find_venv`](crate::virtualenv::venvmanager::VenvManager::find_venv).
pub struct UvVenv<'a> {
    /// The name of the virtual environment (used as the directory name).
    pub name: Cow<'a, str>,
    /// Absolute path to the virtual environment directory.
    pub path: String,
    /// Python version string (e.g. `"3.12"`).
    pub python_version: String,
    /// Packages to install during creation.
    pub packages: Vec<String>,
    /// When `true`, the configured [`default_pkgs`](crate::cfg::settings::Settings::default_pkgs)
    /// are also installed during creation.
    pub default: bool,
    pub settings: settings::Settings,
    /// Number of installed packages (populated on demand by scanning the
    /// environment's `site-packages` directory for `.dist-info` entries).
    pub package_count: Option<usize>,
    /// Sorted list of installed package display strings (`"name version"`).
    pub installed_packages: Vec<String>,
}

impl<'a> Create for UvVenv<'a> {
    async fn create(&self) -> Result<()> {
        // Validate venv name
        Self::validate_venv_name(&self.name)?;

        // Build full path without changing CWD
        let venvs_path = if self.settings.venvs_path.is_empty() {
            DEFAULT_VENV_HOME
        } else {
            &self.settings.venvs_path
        };
        let path = shellexpand::tilde(venvs_path).to_string();

        // Create directory if it doesn't exist
        async_fs::create_dir_all(&path)
            .await
            .map_err(PylotError::Io)?;

        let args = ["venv", &self.name, "--python", self.python_version.as_str()];
        log::info!("Creating virtual environment: {}", self.name);

        // Execute uv venv command in the target directory
        let mut child = tokio::process::Command::new("uv")
            .args(args)
            .current_dir(&path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                PylotError::CommandExecution(format!("Failed to spawn uv command: {}", e))
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PylotError::CommandExecution("Failed to open stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| PylotError::CommandExecution("Failed to open stderr".to_string()))?;

        let stdout_reader = tokio::io::BufReader::new(stdout);
        let stderr_reader = tokio::io::BufReader::new(stderr);

        processes::run_command_with_handlers(
            stdout_reader,
            stderr_reader,
            |line| log::info!("{}", line),
            |line| log::warn!("{}", line),
        )
        .await
        .map_err(|e| PylotError::CommandExecution(e.to_string()))?;

        let mut pkgs = self.packages.clone();
        if self.default {
            let default_pkgs = self.settings.default_pkgs.clone();
            pkgs.extend(default_pkgs);
        }

        if !pkgs.is_empty() {
            // Validate all package names before installation
            for pkg in &pkgs {
                Self::validate_package_name(pkg)?;
            }

            let venv_path = shellexpand::tilde(&self.settings.venvs_path).to_string();
            self.install_packages(pkgs, venv_path).await?;
        }

        Ok(())
    }
}

impl<'a> Delete for UvVenv<'a> {
    async fn delete<R: std::io::Read>(&self, input: R, confirm: bool) -> Result<()> {
        // Validate venv name
        Self::validate_venv_name(&self.name)?;

        let path = shellexpand::tilde(&self.settings.venvs_path).to_string();
        let venv_path = format!("{}/{}", path, self.name);

        if !async_fs::try_exists(&venv_path).await.unwrap_or(false) {
            return Err(PylotError::VenvNotFound(ERROR_VENV_NOT_EXISTS.to_string()));
        }

        let mut choice = !confirm;
        if confirm {
            log::info!(
                "{} {} {} {}",
                "Deleting virtual environment:",
                self.name.red(),
                "at".green(),
                venv_path.replace("\\", "/").red()
            );
            choice = utils::confirm(input);
        }

        if !choice {
            return Ok(());
        }

        async_fs::remove_dir_all(venv_path)
            .await
            .map_err(PylotError::Io)?;

        Ok(())
    }
}

impl<'a> Activate for UvVenv<'a> {
    async fn activate(&self) -> Result<()> {
        // Validate venv name
        Self::validate_venv_name(&self.name)?;

        let (shell, cmd, path) = self.get_shell_cmd()?;

        if !async_fs::try_exists(&path).await.unwrap_or(false) {
            return Err(PylotError::VenvNotFound(ERROR_VENV_NOT_EXISTS.to_string()));
        }

        log::info!("\nActivating virtual environment: {}", self.name);
        log::warn!(
            "{} {}",
            "Note: To exit the virtual environment, type",
            "'exit'".green()
        );

        processes::activate_venv_shell(shell.as_str(), cmd)
            .map_err(|e| PylotError::CommandExecution(e.to_string()))
    }
}

impl<'a> UvVenv<'a> {
    /// Create a new `UvVenv` with the given metadata.
    ///
    /// `path`, `python_version`, `packages`, and `default` may be empty /
    /// `false` when building a lightweight handle used only for name-based
    /// lookups.
    pub fn new(
        name: Cow<'a, str>,
        path: String,
        python_version: String,
        packages: Vec<String>,
        default: bool,
    ) -> Self {
        UvVenv {
            name,
            path,
            python_version,
            packages,
            default,
            settings: settings::Settings::get_settings(),
            package_count: None,
            installed_packages: Vec::new(),
        }
    }

    /// Validates a virtual environment name
    /// Returns an error if the name contains invalid characters
    pub fn validate_venv_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(PylotError::InvalidVenvName(
                "Virtual environment name cannot be empty".to_string(),
            ));
        }

        // Only allow alphanumeric characters, hyphens, and underscores
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(PylotError::InvalidVenvName(format!(
                "Virtual environment name '{}' contains invalid characters. Only alphanumeric, hyphens, and underscores are allowed",
                name
            )));
        }

        Ok(())
    }

    /// Validates a package name to prevent command injection
    /// Returns an error if the name contains potentially dangerous characters
    pub fn validate_package_name(package: &str) -> Result<()> {
        if package.is_empty() {
            return Err(PylotError::InvalidPackageName(
                "Package name cannot be empty".to_string(),
            ));
        }

        // Reject packages with shell metacharacters
        let dangerous_chars = [
            '&', '|', ';', '$', '`', '\n', '\r', '<', '>', '(', ')', '{', '}',
        ];
        if package.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(PylotError::InvalidPackageName(format!(
                "Package name '{}' contains invalid characters",
                package
            )));
        }

        Ok(())
    }

    pub(crate) async fn set_python_version(&mut self) {
        let cfg_path = format!("{}/pyvenv.cfg", self.path);
        if !async_fs::try_exists(&cfg_path).await.unwrap_or(false) {
            return;
        }
        if let Ok(content) = tokio::fs::read_to_string(cfg_path).await {
            for line in content.lines() {
                if line.starts_with("version") {
                    let parts: Vec<&str> = line.split('=').collect();
                    if parts.len() == 2 {
                        self.python_version = parts[1].trim().to_string();
                    }
                }
            }
        }
    }

    /// Scan the venv's `site-packages` for `.dist-info` directories.
    /// Populates `self.package_count` and `self.installed_packages` (sorted).
    pub(crate) async fn count_packages(&mut self) {
        // Unix layout: {path}/lib/pythonX.Y/site-packages/
        if let Ok(mut lib_entries) = async_fs::read_dir(format!("{}/lib", self.path)).await {
            while let Ok(Some(entry)) = lib_entries.next_entry().await {
                if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("python") {
                        let site_pkgs = entry.path().join("site-packages");
                        if let Some(pkgs) = Self::collect_dist_info_packages(&site_pkgs).await {
                            self.package_count = Some(pkgs.len());
                            self.installed_packages = pkgs;
                            return;
                        }
                    }
                }
            }
        }
        // Windows layout: {path}/Lib/site-packages/
        let win_path = std::path::Path::new(&self.path)
            .join("Lib")
            .join("site-packages");
        if let Some(pkgs) = Self::collect_dist_info_packages(&win_path).await {
            self.package_count = Some(pkgs.len());
            self.installed_packages = pkgs;
        }
    }

    /// Collect the names of all installed packages by scanning `.dist-info` directories
    /// inside `site_pkgs`.  Returns `None` if the directory cannot be read.
    async fn collect_dist_info_packages(site_pkgs: &std::path::Path) -> Option<Vec<String>> {
        let mut entries = async_fs::read_dir(site_pkgs).await.ok()?;
        let mut packages = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let is_dist_info = name_str.ends_with(".dist-info");
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            if is_dist_info && is_dir {
                packages.push(Self::format_dist_info_name(&name_str));
            }
        }
        packages.sort_by_key(|a| a.to_lowercase());
        Some(packages)
    }

    /// Convert a `.dist-info` directory name into a human-readable `"name version"` string.
    ///
    /// For example `requests-2.28.0.dist-info` → `"requests 2.28.0"`, and
    /// `Pillow-10.0.0.dist-info` → `"pillow 10.0.0"` (name normalised to lowercase,
    /// underscores replaced with hyphens per PEP 427).
    fn format_dist_info_name(dir_name: &str) -> String {
        let base = dir_name.trim_end_matches(".dist-info");
        if let Some(pos) = base.rfind('-') {
            let name = base[..pos].replace('_', "-").to_lowercase();
            let version = &base[pos + 1..];
            format!("{} {}", name, version)
        } else {
            base.replace('_', "-").to_lowercase()
        }
    }

    /// Add packages to this virtual environment.
    ///
    /// Validates all package names then runs `uv pip install` inside the venv.
    pub async fn add_packages(&self, pkgs: Vec<String>) -> Result<()> {
        if pkgs.is_empty() {
            return Ok(());
        }
        for pkg in &pkgs {
            Self::validate_package_name(pkg)?;
        }
        let venvs_path = shellexpand::tilde(&self.settings.venvs_path).to_string();
        self.install_packages(pkgs, venvs_path).await
    }

    /// Validates a requirements file path to prevent command injection.
    /// Returns an error if the path contains potentially dangerous characters.
    /// Note: backslashes should be normalized to forward slashes before calling this.
    pub fn validate_req_file_path(path: &str) -> Result<()> {
        if path.is_empty() {
            return Err(PylotError::PathError(
                "Requirements file path cannot be empty".to_string(),
            ));
        }
        // Backslashes are accepted as Windows path separators and normalized before this call.
        let dangerous_chars = [
            '&', '|', ';', '$', '`', '\n', '\r', '<', '>', '(', ')', '{', '}', '"', '\'',
        ];
        if path.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(PylotError::PathError(format!(
                "Requirements file path '{}' contains invalid characters",
                path
            )));
        }
        Ok(())
    }

    /// Install packages from a requirements.txt file into this virtual environment.
    ///
    /// Accepts Windows-style backslash paths (normalizes to forward slashes),
    /// validates the path, checks the file exists, then runs `uv pip install -r <req_file>`
    /// inside the venv.
    pub async fn install_from_requirements(&self, req_file: &str) -> Result<()> {
        // Normalize Windows backslashes to forward slashes before validation.
        let normalized = req_file.replace('\\', "/");
        Self::validate_req_file_path(&normalized)?;

        let expanded_path = shellexpand::tilde(&normalized).to_string();

        if !async_fs::try_exists(&expanded_path).await.unwrap_or(false) {
            return Err(PylotError::PathError(format!(
                "Requirements file not found: {}",
                expanded_path
            )));
        }

        log::info!("Installing from requirements file: {}", expanded_path);

        let venvs_path = shellexpand::tilde(&self.settings.venvs_path).to_string();
        let activate_script = if cfg!(target_os = "windows") {
            format!("{}/{}/scripts/activate.ps1", venvs_path, self.name)
        } else {
            format!("{}/{}/bin/activate", venvs_path, self.name)
        };

        let (cmd, args) = if cfg!(target_os = "windows") {
            let pwsh_cmd = if uvctrl::check(PWSH_CMD).await.is_ok() {
                PWSH_CMD
            } else {
                POWERSHELL_CMD
            };
            let command = format!(
                "{} ; uv pip install -r \"{}\"",
                activate_script, expanded_path
            );
            (pwsh_cmd, vec!["-Command".to_string(), command])
        } else {
            let command = format!(
                ". {} && uv pip install -r \"{}\"",
                activate_script, expanded_path
            );
            (SH_CMD, vec!["-c".to_string(), command])
        };

        let mut child = tokio::process::Command::new(cmd)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                PylotError::CommandExecution(format!("Failed to install from requirements: {}", e))
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PylotError::CommandExecution("Failed to open stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| PylotError::CommandExecution("Failed to open stderr".to_string()))?;

        let stdout_reader = tokio::io::BufReader::new(stdout);
        let stderr_reader = tokio::io::BufReader::new(stderr);

        processes::run_command_with_handlers(
            stdout_reader,
            stderr_reader,
            |line| log::info!("{}", line),
            |line| log::warn!("{}", line),
        )
        .await
        .map_err(|e| {
            PylotError::CommandExecution(format!("Error installing from requirements: {}", e))
        })?;

        Ok(())
    }

    /// Remove packages from this virtual environment.
    ///
    /// Validates all package names then runs `uv pip uninstall` inside the venv.
    pub async fn remove_packages(&self, pkgs: Vec<String>) -> Result<()> {
        if pkgs.is_empty() {
            return Ok(());
        }
        for pkg in &pkgs {
            Self::validate_package_name(pkg)?;
        }
        let venvs_path = shellexpand::tilde(&self.settings.venvs_path).to_string();
        self.uninstall_packages(pkgs, venvs_path).await
    }

    /// Uninstall packages without shell command injection
    async fn uninstall_packages(&self, pkgs: Vec<String>, venv_path: String) -> Result<()> {
        log::info!("{} {}", "Uninstalling package(s):", pkgs.join(", "));

        let activate_script = if cfg!(target_os = "windows") {
            format!("{}/{}/scripts/activate.ps1", venv_path, self.name)
        } else {
            format!("{}/{}/bin/activate", venv_path, self.name)
        };

        let (cmd, args) = if cfg!(target_os = "windows") {
            let pwsh_cmd = if uvctrl::check(PWSH_CMD).await.is_ok() {
                PWSH_CMD
            } else {
                POWERSHELL_CMD
            };

            let mut command_parts = vec![activate_script.clone(), ";".to_string()];
            command_parts.push("uv".to_string());
            command_parts.push("pip".to_string());
            command_parts.push("uninstall".to_string());
            command_parts.extend(pkgs.iter().cloned());

            (
                pwsh_cmd,
                vec!["-Command".to_string(), command_parts.join(" ")],
            )
        } else {
            let mut command_parts =
                vec![".".to_string(), activate_script.clone(), "&&".to_string()];
            command_parts.push("uv".to_string());
            command_parts.push("pip".to_string());
            command_parts.push("uninstall".to_string());
            command_parts.extend(pkgs.iter().cloned());

            (SH_CMD, vec!["-c".to_string(), command_parts.join(" ")])
        };

        let mut child = tokio::process::Command::new(cmd)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                PylotError::CommandExecution(format!("Failed to uninstall packages: {}", e))
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PylotError::CommandExecution("Failed to open stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| PylotError::CommandExecution("Failed to open stderr".to_string()))?;

        let stdout_reader = tokio::io::BufReader::new(stdout);
        let stderr_reader = tokio::io::BufReader::new(stderr);

        processes::run_command_with_handlers(
            stdout_reader,
            stderr_reader,
            |line| log::info!("{}", line),
            |line| log::warn!("{}", line),
        )
        .await
        .map_err(|e| PylotError::CommandExecution(format!("Error uninstalling packages: {}", e)))?;

        Ok(())
    }

    /// Install packages without shell command injection
    async fn install_packages(&self, pkgs: Vec<String>, venv_path: String) -> Result<()> {
        log::info!("{} {}", "Installing package(s):", pkgs.join(", "));

        // Determine activation script path
        let activate_script = if cfg!(target_os = "windows") {
            format!("{}/{}/scripts/activate.ps1", venv_path, self.name)
        } else {
            format!("{}/{}/bin/activate", venv_path, self.name)
        };

        // Build command to activate venv and install packages
        // Note: We must concatenate packages into a single command string because both
        // PowerShell's -Command and sh's -c require a single string argument.
        // Package names are validated beforehand to prevent injection attacks.
        let (cmd, args) = if cfg!(target_os = "windows") {
            let pwsh_cmd = if uvctrl::check(PWSH_CMD).await.is_ok() {
                PWSH_CMD
            } else {
                POWERSHELL_CMD
            };

            // For PowerShell, we need to build a command string that activates and then runs uv
            let mut command_parts = vec![activate_script.clone(), ";".to_string()];
            command_parts.push("uv".to_string());
            command_parts.push("pip".to_string());
            command_parts.push("install".to_string());
            command_parts.extend(pkgs.iter().cloned());

            (
                pwsh_cmd,
                vec!["-Command".to_string(), command_parts.join(" ")],
            )
        } else {
            // For Unix, we use command chaining with sh -c
            let mut command_parts =
                vec![".".to_string(), activate_script.clone(), "&&".to_string()];
            command_parts.push("uv".to_string());
            command_parts.push("pip".to_string());
            command_parts.push("install".to_string());
            command_parts.extend(pkgs.iter().cloned());

            (SH_CMD, vec!["-c".to_string(), command_parts.join(" ")])
        };

        let mut child = tokio::process::Command::new(cmd)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                PylotError::CommandExecution(format!("Failed to install packages: {}", e))
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PylotError::CommandExecution("Failed to open stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| PylotError::CommandExecution("Failed to open stderr".to_string()))?;

        let stdout_reader = tokio::io::BufReader::new(stdout);
        let stderr_reader = tokio::io::BufReader::new(stderr);

        processes::run_command_with_handlers(
            stdout_reader,
            stderr_reader,
            |line| log::info!("{}", line),
            |line| log::warn!("{}", line),
        )
        .await
        .map_err(|e| PylotError::CommandExecution(format!("Error installing packages: {}", e)))?;

        Ok(())
    }

    fn get_shell_cmd(&self) -> Result<(String, Vec<String>, String)> {
        // Validate venv name to prevent command injection
        Self::validate_venv_name(&self.name)?;

        let path = shellexpand::tilde(&self.settings.venvs_path).to_string();
        let shell = processes::get_parent_shell()?;

        let (cmd, path) = if cfg!(target_os = "windows") {
            let venv_path = format!("{}/{}/scripts/activate.ps1", path, self.name);
            // Use -NoExit so the shell stays open after activating, and -Command with the
            // call operator (&) to execute the activation script correctly.
            let activate_cmd = format!("& '{}'", venv_path);
            (
                vec!["-NoExit".to_string(), "-Command".to_string(), activate_cmd],
                venv_path,
            )
        } else {
            let venv_path = format!("{}/{}/bin/activate", path, self.name);
            // Return the command string to execute. The -c flag will be added by activate_venv_shell.
            // This constructs a command that sources the activation script and starts an interactive shell.
            let venv_cmd = format!(". {} && {} -i", venv_path, shell.as_str());
            (vec![venv_cmd], venv_path)
        };

        Ok((shell, cmd, path))
    }
}

#[cfg(test)]
mod tests {
    use crate::logger;

    use super::*;

    #[tokio::test]
    async fn test_validate_venv_name_valid() {
        logger::initialize_logger(log::LevelFilter::Trace);
        assert!(UvVenv::validate_venv_name("test_venv").is_ok());
        assert!(UvVenv::validate_venv_name("test-venv").is_ok());
        assert!(UvVenv::validate_venv_name("test123").is_ok());
        assert!(UvVenv::validate_venv_name("TestVenv").is_ok());
    }

    #[tokio::test]
    async fn test_validate_venv_name_invalid() {
        logger::initialize_logger(log::LevelFilter::Trace);
        assert!(UvVenv::validate_venv_name("").is_err());
        assert!(UvVenv::validate_venv_name("test/venv").is_err());
        assert!(UvVenv::validate_venv_name("test venv").is_err());
        assert!(UvVenv::validate_venv_name("../../etc").is_err());
        assert!(UvVenv::validate_venv_name("test;venv").is_err());
    }

    #[tokio::test]
    async fn test_validate_package_name_valid() {
        logger::initialize_logger(log::LevelFilter::Trace);
        assert!(UvVenv::validate_package_name("requests").is_ok());
        assert!(UvVenv::validate_package_name("numpy==1.20.0").is_ok());
        assert!(UvVenv::validate_package_name("flask-restful").is_ok());
    }

    #[tokio::test]
    async fn test_validate_package_name_invalid() {
        logger::initialize_logger(log::LevelFilter::Trace);
        assert!(UvVenv::validate_package_name("").is_err());
        assert!(UvVenv::validate_package_name("numpy; rm -rf /").is_err());
        assert!(UvVenv::validate_package_name("test && evil").is_err());
        assert!(UvVenv::validate_package_name("test | cat").is_err());
        assert!(UvVenv::validate_package_name("test$variable").is_err());
    }

    #[tokio::test]
    async fn test_venv() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = UvVenv::new(
            Cow::Borrowed("test_venv"),
            "".to_string(),
            "3.8".to_string(),
            vec![],
            false,
        );
        assert_eq!(venv.name, "test_venv");
        assert_eq!(venv.python_version, "3.8");
    }

    #[tokio::test]
    async fn test_venv_clean() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = UvVenv::new(
            Cow::Borrowed("test_venv_clean"),
            "".to_string(),
            "3.9".to_string(),
            vec!["numpy".to_string(), "pandas".to_string()],
            false,
        );
        assert_eq!(venv.name, "test_venv_clean");
        assert_eq!(venv.python_version, "3.9");
        assert_eq![venv.packages, &["numpy", "pandas"]]
    }

    #[tokio::test]
    async fn test_get_shell_cmd_windows() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = UvVenv::new(
            Cow::Borrowed("test_venv"),
            "".to_string(),
            "3.9".to_string(),
            vec![],
            false,
        );
        if cfg!(target_os = "windows") {
            let result = venv.get_shell_cmd();
            assert!(result.is_ok());
            let (_shell, cmd, path) = result.unwrap();
            // Verify that args use -NoExit and -Command instead of a combined path string
            assert_eq!(cmd.len(), 3);
            assert_eq!(cmd[0], "-NoExit");
            assert_eq!(cmd[1], "-Command");
            assert!(cmd[2].starts_with("& '"));
            assert!(cmd[2].ends_with("activate.ps1'"));
            assert!(path.ends_with("activate.ps1"));
        } else {
            let result = venv.get_shell_cmd();
            assert!(result.is_ok());
            let (_shell, cmd, path) = result.unwrap();
            assert_eq!(cmd.len(), 1);
            assert!(cmd[0].contains("activate"));
            assert!(path.ends_with("activate"));
        }
    }

    // ── set_python_version ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_set_python_version_reads_cfg_file() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("pyvenv.cfg");
        tokio::fs::write(&cfg_path, "home = /usr/bin\nversion = 3.11.2\n")
            .await
            .unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("myenv"),
            dir.path().to_str().unwrap().to_string(),
            "".to_string(),
            vec![],
            false,
        );
        venv.set_python_version().await;
        assert_eq!(venv.python_version, "3.11.2");
    }

    #[tokio::test]
    async fn test_set_python_version_no_cfg_file() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("myenv"),
            dir.path().to_str().unwrap().to_string(),
            "original".to_string(),
            vec![],
            false,
        );
        // No pyvenv.cfg exists – version should remain unchanged.
        venv.set_python_version().await;
        assert_eq!(venv.python_version, "original");
    }

    #[tokio::test]
    async fn test_set_python_version_cfg_without_version_key() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("pyvenv.cfg");
        tokio::fs::write(
            &cfg_path,
            "home = /usr/bin\ninclude-system-site-packages = false\n",
        )
        .await
        .unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("myenv"),
            dir.path().to_str().unwrap().to_string(),
            "fallback".to_string(),
            vec![],
            false,
        );
        venv.set_python_version().await;
        // No "version" key → version stays unchanged.
        assert_eq!(venv.python_version, "fallback");
    }

    // ── validate_package_name – individual dangerous characters ──────────────

    #[test]
    fn test_validate_package_name_each_dangerous_char() {
        logger::initialize_logger(log::LevelFilter::Trace);
        for ch in [
            '&', '|', ';', '$', '`', '\n', '\r', '<', '>', '(', ')', '{', '}',
        ] {
            let pkg = format!("pkg{}name", ch);
            assert!(
                UvVenv::validate_package_name(&pkg).is_err(),
                "Expected error for package name containing '{}'",
                ch
            );
        }
    }

    // ── validate_venv_name – additional edge cases ────────────────────────────

    #[test]
    fn test_validate_venv_name_dot_rejected() {
        logger::initialize_logger(log::LevelFilter::Trace);
        assert!(UvVenv::validate_venv_name("my.env").is_err());
    }

    #[test]
    fn test_validate_venv_name_space_rejected() {
        logger::initialize_logger(log::LevelFilter::Trace);
        assert!(UvVenv::validate_venv_name("my env").is_err());
    }

    // ── count_packages ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_count_packages_no_site_packages_dir() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("myenv"),
            dir.path().to_str().unwrap().to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        // No site-packages directory → package_count stays None, list empty.
        venv.count_packages().await;
        assert_eq!(venv.package_count, None);
        assert!(venv.installed_packages.is_empty());
    }

    #[tokio::test]
    async fn test_count_packages_unix_layout() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();

        // Create a fake Unix venv layout with two .dist-info dirs and one non-dist-info dir.
        let site_pkgs = dir
            .path()
            .join("lib")
            .join("python3.11")
            .join("site-packages");
        tokio::fs::create_dir_all(&site_pkgs).await.unwrap();
        tokio::fs::create_dir_all(site_pkgs.join("requests-2.28.0.dist-info"))
            .await
            .unwrap();
        tokio::fs::create_dir_all(site_pkgs.join("flask-3.0.0.dist-info"))
            .await
            .unwrap();
        tokio::fs::create_dir_all(site_pkgs.join("requests"))
            .await
            .unwrap(); // not a dist-info dir

        let mut venv = UvVenv::new(
            Cow::Borrowed("myenv"),
            dir.path().to_str().unwrap().to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        venv.count_packages().await;
        assert_eq!(venv.package_count, Some(2));
        assert_eq!(venv.installed_packages.len(), 2);
        // Sorted alphabetically: flask before requests
        assert_eq!(venv.installed_packages[0], "flask 3.0.0");
        assert_eq!(venv.installed_packages[1], "requests 2.28.0");
    }

    #[tokio::test]
    async fn test_count_packages_windows_layout() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();

        // Create a fake Windows venv layout.
        let site_pkgs = dir.path().join("Lib").join("site-packages");
        tokio::fs::create_dir_all(&site_pkgs).await.unwrap();
        tokio::fs::create_dir_all(site_pkgs.join("numpy-1.26.0.dist-info"))
            .await
            .unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("myenv"),
            dir.path().to_str().unwrap().to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        venv.count_packages().await;
        // On Linux the Unix layout search will find no "python*" dir, so it
        // falls back to the Windows layout.
        assert_eq!(venv.package_count, Some(1));
        assert_eq!(venv.installed_packages, vec!["numpy 1.26.0"]);
    }

    // ── format_dist_info_name ────────────────────────────────────────────────

    #[test]
    fn test_format_dist_info_name_simple() {
        assert_eq!(
            UvVenv::format_dist_info_name("requests-2.28.0.dist-info"),
            "requests 2.28.0"
        );
    }

    #[test]
    fn test_format_dist_info_name_underscores() {
        // Underscores in the package name are replaced with hyphens (PEP 427).
        assert_eq!(
            UvVenv::format_dist_info_name("my_package-1.0.0.dist-info"),
            "my-package 1.0.0"
        );
    }

    #[test]
    fn test_format_dist_info_name_uppercase() {
        // Name is normalised to lowercase.
        assert_eq!(
            UvVenv::format_dist_info_name("Pillow-10.0.0.dist-info"),
            "pillow 10.0.0"
        );
    }

    #[test]
    fn test_format_dist_info_name_no_version() {
        // No `-` separator → treat the whole thing as the name.
        assert_eq!(
            UvVenv::format_dist_info_name("somepkg.dist-info"),
            "somepkg"
        );
    }

    // ── add_packages / remove_packages (validation) ──────────────────────────

    #[tokio::test]
    async fn test_add_packages_empty_list_returns_ok() {
        let venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        // Empty list should short-circuit without attempting any subprocess.
        assert!(venv.add_packages(vec![]).await.is_ok());
    }

    #[tokio::test]
    async fn test_remove_packages_empty_list_returns_ok() {
        let venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        assert!(venv.remove_packages(vec![]).await.is_ok());
    }

    #[tokio::test]
    async fn test_add_packages_rejects_invalid_package_name() {
        let venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        let result = venv.add_packages(vec!["evil; rm -rf /".to_string()]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_packages_rejects_invalid_package_name() {
        let venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        let result = venv.remove_packages(vec!["evil | cat".to_string()]).await;
        assert!(result.is_err());
    }

    // ── validate_req_file_path ────────────────────────────────────────────────

    #[test]
    fn test_validate_req_file_path_empty_is_error() {
        assert!(UvVenv::validate_req_file_path("").is_err());
    }

    #[test]
    fn test_validate_req_file_path_valid_unix() {
        assert!(UvVenv::validate_req_file_path("/home/user/requirements.txt").is_ok());
    }

    #[test]
    fn test_validate_req_file_path_valid_windows_normalized() {
        // Backslashes are normalized to forward slashes before validation.
        let normalized = "C:/Users/user/requirements.txt";
        assert!(UvVenv::validate_req_file_path(normalized).is_ok());
    }

    #[test]
    fn test_validate_req_file_path_rejects_injection_chars() {
        assert!(UvVenv::validate_req_file_path("path;evil").is_err());
        assert!(UvVenv::validate_req_file_path("path|evil").is_err());
        assert!(UvVenv::validate_req_file_path("path&evil").is_err());
        assert!(UvVenv::validate_req_file_path("path$evil").is_err());
    }

    // ── delete ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_delete_nonexistent_venv_returns_not_found() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("nonexistent_venv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        // Point settings at temp dir; venv subdirectory does not exist.
        venv.settings.venvs_path = dir.path().to_str().unwrap().to_string();

        let result = venv.delete(std::io::Cursor::new(""), false).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PylotError::VenvNotFound(_)));
    }

    #[tokio::test]
    async fn test_delete_venv_invalid_name_rejected() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let mut venv = UvVenv::new(
            Cow::Borrowed("bad/name"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        venv.settings.venvs_path = "/tmp".to_string();

        let result = venv.delete(std::io::Cursor::new(""), false).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PylotError::InvalidVenvName(_)
        ));
    }

    #[tokio::test]
    async fn test_delete_venv_user_confirms_no() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();
        let venv_dir = dir.path().join("myvenv");
        tokio::fs::create_dir_all(&venv_dir).await.unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        venv.settings.venvs_path = dir.path().to_str().unwrap().to_string();

        // confirm=true but user answers "n" → deletion skipped.
        let result = venv.delete(std::io::Cursor::new("n\n"), true).await;
        assert!(result.is_ok());
        // Directory must still exist.
        assert!(tokio::fs::try_exists(&venv_dir).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_venv_no_confirm_removes_directory() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();
        let venv_dir = dir.path().join("myvenv");
        tokio::fs::create_dir_all(&venv_dir).await.unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        venv.settings.venvs_path = dir.path().to_str().unwrap().to_string();

        // confirm=false → skip prompt, delete immediately.
        let result = venv.delete(std::io::Cursor::new(""), false).await;
        assert!(result.is_ok());
        // Directory should have been removed.
        assert!(!tokio::fs::try_exists(&venv_dir).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_venv_user_confirms_yes() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();
        let venv_dir = dir.path().join("myvenv2");
        tokio::fs::create_dir_all(&venv_dir).await.unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("myvenv2"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        venv.settings.venvs_path = dir.path().to_str().unwrap().to_string();

        // confirm=true and user answers "y" → directory deleted.
        let result = venv.delete(std::io::Cursor::new("y\n"), true).await;
        assert!(result.is_ok());
        assert!(!tokio::fs::try_exists(&venv_dir).await.unwrap());
    }

    // ── install_from_requirements ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_install_from_requirements_empty_path_is_error() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        let result = venv.install_from_requirements("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_install_from_requirements_injection_path_is_error() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        let result = venv.install_from_requirements("/tmp/evil;cmd").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_install_from_requirements_nonexistent_file_is_error() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        let result = venv
            .install_from_requirements("/tmp/nonexistent_req_xyz_abc_123.txt")
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PylotError::PathError(_)));
    }

    #[tokio::test]
    async fn test_install_from_requirements_backslash_path_normalized() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        // Backslash path is valid syntax but file does not exist → PathError (not validation error).
        let result = venv
            .install_from_requirements("C:\\Users\\user\\nonexistent_req.txt")
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PylotError::PathError(_)));
    }

    // ── activate ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_activate_invalid_name_is_error() {
        logger::initialize_logger(log::LevelFilter::Trace);
        let mut venv = UvVenv::new(
            Cow::Borrowed("bad/name"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        venv.settings.venvs_path = "/tmp".to_string();
        let result = venv.activate().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_activate_nonexistent_venv_is_not_found() {
        use tempfile::tempdir;

        logger::initialize_logger(log::LevelFilter::Trace);
        let dir = tempdir().unwrap();

        let mut venv = UvVenv::new(
            Cow::Borrowed("myvenv"),
            "".to_string(),
            "3.11".to_string(),
            vec![],
            false,
        );
        // Point settings at temp dir; the venv (and its activate script) do not exist.
        venv.settings.venvs_path = dir.path().to_str().unwrap().to_string();

        let result = venv.activate().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PylotError::VenvNotFound(_)));
    }
}
