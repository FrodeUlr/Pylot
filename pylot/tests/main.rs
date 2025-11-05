#[cfg(test)]
mod tests {
    use clap::Parser;

    use pylot::cli::cmds::{Cli, Commands, VenvCommands};
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
            .with_args(&["uv", "check"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stderr()
            .contains("Checking if Astral UV is installed and configured...")
            .unwrap();
    }

    #[test]
    fn test_cli_output_activate() {
        assert_cli::Assert::main_binary()
            .with_args(&["venv", "activate"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stderr()
            .contains("virtual environment")
            .unwrap();
    }

    #[test]
    fn test_cli_output_delete() {
        assert_cli::Assert::main_binary()
            .with_args(&["venv", "delete"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stderr()
            .contains("virtual environment")
            .unwrap();
    }

    #[test]
    fn test_cli_output_delete_name() {
        assert_cli::Assert::main_binary()
            .with_args(&["venv", "delete", "myvenv"])
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
            .with_args(&["venv", "activate", "myvenv"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .succeeds()
            .and()
            .stderr()
            .contains(ERROR_VENV_NOT_EXISTS)
            .unwrap();
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
}
