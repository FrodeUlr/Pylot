mod cli;

use pylot::{activate, check, create, delete, install, list, uninstall};
use std::io;

use clap::Parser;
use cli::cmds::{Cli, Commands};
use shared::settings;

#[tokio::main]
async fn main() {
    settings::Settings::init().await;
    let _ = color_eyre::install();
    let args = Cli::parse();

    match args.commands {
        Some(Commands::Activate { name_pos, name }) => activate(name_pos, name).await,

        Some(Commands::Check) => check().await,

        Some(Commands::Create {
            name_pos,
            name,
            python_version,
            packages,
            requirements,
            default,
        }) => {
            create(
                name_pos,
                name,
                python_version,
                packages,
                requirements,
                default,
            )
            .await
        }

        Some(Commands::Delete { name_pos, name }) => delete(io::stdin(), name_pos, name).await,

        Some(Commands::List) => list().await,

        Some(Commands::Install { update }) => install(io::stdin(), update).await,

        Some(Commands::Uninstall) => uninstall(io::stdin()).await,

        Some(Commands::Tui) => {
            _ = tui::run::run();
        }

        None => {
            println!("No command provided");
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::cmds::{Cli, Commands};
    use shared::constants::ERROR_VENV_NOT_EXISTS;

    #[test]
    fn test_cli_output_help() {
        assert_cli::Assert::main_binary()
            .with_args(&["--help"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stdout()
            .contains("A simple CLI to manage Python virtual enviroonments using Astral UV")
            .unwrap();
    }

    #[test]
    fn test_cli_output_version() {
        let version = env!("CARGO_PKG_VERSION");
        assert_cli::Assert::main_binary()
            .with_args(&["--version"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stdout()
            .contains(format!("pylot {}", version).as_str())
            .unwrap();
    }

    #[test]
    fn test_cli_output_check() {
        assert_cli::Assert::main_binary()
            .with_args(&["check"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stdout()
            .contains("Checking if Astral UV is installed and configured...")
            .unwrap();
    }

    #[test]
    fn test_cli_output_activate() {
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        assert_cli::Assert::main_binary()
            .with_args(&["activate"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stdout()
            .contains("No virtual environments found")
            .unwrap();
    }

    #[test]
    fn test_cli_output_delete() {
        if std::env::var("GITHUB_ACTIONS").is_err() {
            println!("Skipping test in non-GitHub Actions environment");
            return;
        }
        assert_cli::Assert::main_binary()
            .with_args(&["delete"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stdout()
            .contains("No virtual environments found")
            .unwrap();
    }

    #[test]
    fn test_cli_output_delete_name() {
        assert_cli::Assert::main_binary()
            .with_args(&["delete", "myvenv"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stderr()
            .contains(ERROR_VENV_NOT_EXISTS)
            .unwrap();
    }

    #[test]
    fn test_cli_output_activate_name() {
        assert_cli::Assert::main_binary()
            .with_args(&["activate", "myvenv"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stderr()
            .contains(ERROR_VENV_NOT_EXISTS)
            .unwrap();
    }

    #[test]
    fn test_activate_command() {
        let args = Cli::try_parse_from(["program", "activate", "my-venv"]).unwrap();

        match args.commands {
            Some(Commands::Activate { name_pos, name }) => {
                assert_eq!(name_pos, Some("my-venv".to_string()));
                assert_eq!(name, None);
            }
            _ => panic!("Expected Activate command"),
        }
    }

    #[test]
    fn test_activate_with_flag() {
        let args = Cli::try_parse_from(["program", "activate", "--name", "my-venv"]).unwrap();

        match args.commands {
            Some(Commands::Activate { name_pos, name }) => {
                assert_eq!(name_pos, None);
                assert_eq!(name, Some("my-venv".to_string()));
            }
            _ => panic!("Expected Activate command"),
        }
    }

    #[test]
    fn test_create_command() {
        let args = Cli::try_parse_from([
            "program",
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

        match args.commands {
            Some(Commands::Create {
                name_pos,
                python_version,
                packages,
                default,
                ..
            }) => {
                assert_eq!(name_pos, Some("my-venv".to_string()));
                assert_eq!(python_version, "3.11");
                assert_eq!(packages, vec!["requests", "numpy"]);
                assert!(default);
            }
            _ => panic!("Expected Create command"),
        }
    }

    #[test]
    fn test_list_command() {
        let args = Cli::try_parse_from(["program", "list"]).unwrap();

        assert!(matches!(args.commands, Some(Commands::List)));
    }

    #[test]
    fn test_no_command() {
        let result = Cli::try_parse_from(["program"]);

        assert!(result.is_err());
    }
}
