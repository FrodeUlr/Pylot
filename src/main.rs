mod cfg;
mod cli;
mod core;
mod shell;
mod utility;

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
    use crate::utility::constants::ERROR_VENV_NOT_EXISTS;

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
    #[ignore = "Only run in github actions"]
    fn test_cli_output_activate() {
        assert_cli::Assert::main_binary()
            .with_args(&["activate"])
            .succeeds()
            .and()
            .stdout()
            .contains("No virtual environments found")
            .unwrap();
    }

    #[test]
    #[ignore = "Only run in github actions"]
    fn test_cli_output_delete() {
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
}
