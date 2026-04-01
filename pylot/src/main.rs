pub mod cli;

use clap_complete::{generate, Shell};
use pylot::{activate, check, create, delete, install, list, uninstall, update};
use std::{io, str::FromStr};

use clap::{CommandFactory, Parser};
use cli::cmds::{Cli, Commands};
use pylot_shared::{logger, settings};

use crate::cli::cmds::{UvCommands, VenvCommands};

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    // Handle completion generation before settings/logger init so that no
    // diagnostic messages are written to stdout and pollute the script output.
    if let Some(Commands::Complete { shell }) = &args.commands {
        let shell = shell
            .as_deref()
            .and_then(|s| Shell::from_str(s).ok())
            .unwrap_or(Shell::Bash);
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "pylot", &mut io::stdout());
        return;
    }

    settings::Settings::init().await;
    logger::initialize_logger(log::LevelFilter::Info);

    match args.commands {
        Some(Commands::Complete { .. }) => unreachable!(),
        Some(Commands::Uv { command }) => match command {
            UvCommands::Install => match install(io::stdin()).await {
                Ok(_) => {}
                Err(e) => log::error!("{}", e),
            },
            UvCommands::Update => update().await,
            UvCommands::Uninstall => match uninstall(io::stdin()).await {
                Ok(_) => {}
                Err(e) => log::error!("{}", e),
            },
            UvCommands::Check => match check().await {
                Ok(_) => log::info!("Astral UV is installed"),
                Err(e) => log::error!("{}", e),
            },
        },

        Some(Commands::Venv { command }) => match command {
            VenvCommands::Activate { name_pos, name } => {
                let venv_name = name.or(name_pos);
                match activate(venv_name.as_deref()).await {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error activating environment: {}", e);
                    }
                }
            }
            VenvCommands::Create {
                name_pos,
                name,
                python_version,
                packages,
                requirements,
                default,
            } => {
                let name = match name.or(name_pos) {
                    Some(n) => n,
                    None => {
                        log::error!("Virtual environment name is required for creation");
                        return;
                    }
                };
                match create(
                    &name,
                    Some(&python_version),
                    Some(packages),
                    Some(&requirements),
                    default,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }
            VenvCommands::Delete { name_pos, name } => {
                let venv_name = name.or(name_pos);
                match delete(io::stdin(), io::stdin(), venv_name.as_deref()).await {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error deleting environment: {}", e);
                    }
                }
            }
            VenvCommands::List => list().await,
        },

        None => {
            log::error!("No command provided");
        }

        Some(Commands::Tui) => {
            if let Err(e) = pylot_tui::run().await {
                log::error!("TUI error: {}", e);
            }
        }
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use assert_cmd::Command;
    use clap::Parser;
    use predicates::prelude::*;

    use pylot::cli::cmds::{Cli, Commands, VenvCommands};
    use pylot_shared::constants::ERROR_VENV_NOT_EXISTS;

    #[test]
    fn test_cli_output_help() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.arg("--help")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "A simple CLI to manage Python virtual enviroonments using Astral UV",
            ));
    }

    #[test]
    fn test_cli_output_version() {
        let version = env!("CARGO_PKG_VERSION");
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.arg("--version")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("pylot {}", version)));
    }

    #[test]
    fn test_cli_output_check() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(["uv", "check"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .assert()
            .success()
            .stderr(predicate::str::contains(
                "Checking if Astral UV is installed and configured...",
            ));
    }

    #[test]
    fn test_cli_output_activate() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(["venv", "activate"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .assert()
            .success()
            .stderr(predicate::str::contains("virtual environment"));
    }

    #[test]
    fn test_cli_output_delete() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(["venv", "delete"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .assert()
            .success()
            .stderr(predicate::str::contains("virtual environment"));
    }

    #[test]
    fn test_cli_output_delete_name() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(["venv", "delete", "myvenv"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .assert()
            .success()
            .stderr(predicate::str::contains(ERROR_VENV_NOT_EXISTS));
    }

    #[test]
    fn test_cli_output_activate_name() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(["venv", "activate", "myvenv"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .assert()
            .success()
            .stderr(predicate::str::contains(ERROR_VENV_NOT_EXISTS));
    }

    #[test]
    fn test_activate_command() {
        let args = Cli::try_parse_from(["program", "venv", "activate", "my-venv"]).unwrap();

        if let Some(Commands::Venv {
            command: VenvCommands::Activate { name_pos, name },
        }) = args.commands
        {
            assert_eq!(name_pos, Some("my-venv".to_string()));
            assert_eq!(name, None);
        } else {
            panic!("Failed to parse activate command");
        }
    }

    #[test]
    fn test_activate_with_flag() {
        let args =
            Cli::try_parse_from(["program", "venv", "activate", "--name", "my-venv"]).unwrap();

        if let Some(Commands::Venv {
            command: VenvCommands::Activate { name_pos, name },
        }) = args.commands
        {
            assert_eq!(name, Some("my-venv".to_string()));
            assert_eq!(name_pos, None);
        } else {
            panic!("Failed to parse activate command with flag");
        }
    }

    #[test]
    fn test_create_command() {
        let args = Cli::try_parse_from([
            "program",
            "venv",
            "create",
            "my-venv",
            "--python-version",
            "3.11",
            "--packages",
            "requests",
            "numpy",
            "--default",
        ])
        .unwrap();

        if let Some(Commands::Venv {
            command:
                VenvCommands::Create {
                    name_pos,
                    python_version,
                    packages,
                    default,
                    ..
                },
        }) = args.commands
        {
            assert_eq!(name_pos, Some("my-venv".to_string()));
            assert_eq!(python_version, "3.11");
            assert_eq!(packages, vec!["requests", "numpy"]);
            assert!(default);
        } else {
            panic!("Failed to parse create command");
        }
    }

    #[test]
    fn test_list_command() {
        let args = Cli::try_parse_from(["program", "venv", "list"]).unwrap();

        assert!(matches!(
            args.commands,
            Some(Commands::Venv {
                command: VenvCommands::List
            })
        ));
    }

    #[test]
    fn test_no_command() {
        let result = Cli::try_parse_from(["program"]);

        assert!(result.is_err());
    }

    #[test]
    fn test_complete_bash() {
        let args = Cli::try_parse_from(["program", "complete", "bash"]).unwrap();

        if let Some(Commands::Complete { shell }) = args.commands {
            assert_eq!(shell, Some("bash".to_string()));
        } else {
            panic!("Failed to parse complete command for bash");
        }
    }
    #[test]
    fn test_complete_zsh() {
        let args = Cli::try_parse_from(["program", "complete", "zsh"]).unwrap();

        if let Some(Commands::Complete { shell }) = args.commands {
            assert_eq!(shell, Some("zsh".to_string()));
        } else {
            panic!("Failed to parse complete command for zsh");
        }
    }
    #[test]
    fn test_complete_powershell() {
        let args = Cli::try_parse_from(["program", "complete", "powershell"]).unwrap();

        if let Some(Commands::Complete { shell }) = args.commands {
            assert_eq!(shell, Some("powershell".to_string()));
        } else {
            panic!("Failed to parse complete command for powershell");
        }
    }

    /// Verifies that `pylot complete powershell` outputs only the completion
    /// script to stdout, with no diagnostic messages prepended.  PowerShell
    /// requires `using` statements to be the very first lines of a script, so
    /// stdout must not contain any diagnostic messages before "using namespace".
    #[test]
    fn test_complete_powershell_output_starts_with_using() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args(["complete", "powershell"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .assert()
            .success()
            .stdout(predicate::str::contains("Settings.toml is invalid, using defaults").not())
            .stdout(predicate::str::contains("Creating venvs folder").not())
            .stdout(predicate::str::is_match(r"^\s*using namespace").unwrap());
    }
}
