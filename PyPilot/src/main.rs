mod cfg;
mod cli;
mod core;

use crate::cli::run;
use cfg::settings;
use clap::Parser;
use cli::clicmd::{Cli, Commands};

#[tokio::main]
async fn main() {
    settings::Settings::init().await;
    let args = Cli::parse();

    match args.commands {
        Some(Commands::Activate { name_pos, name }) => run::activate(name_pos, name).await,

        Some(Commands::Check) => run::check().await,

        Some(Commands::Create {
            name_pos,
            name,
            python_version,
            packages,
            requirements,
            default,
        }) => {
            run::create(
                name_pos,
                name,
                python_version,
                packages,
                requirements,
                default,
            )
            .await
        }

        Some(Commands::Delete { name_pos, name }) => run::delete(name_pos, name).await,

        Some(Commands::List) => run::list().await,

        Some(Commands::Install { update }) => run::install(update).await,

        Some(Commands::Uninstall) => run::uninstall().await,

        None => {
            println!("No command provided");
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::clicmd::{Cli, Commands};
    use pypilotlib::constants::ERROR_VENV_NOT_EXISTS;

    #[test]
    fn test_cli_output_help() {
        assert_cli::Assert::main_binary()
            .with_args(&["--help"])
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
            .succeeds()
            .and()
            .stdout()
            .contains(format!("PyPilot {}", version).as_str())
            .unwrap();
    }

    #[test]
    fn test_cli_output_check() {
        assert_cli::Assert::main_binary()
            .with_args(&["check"])
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
