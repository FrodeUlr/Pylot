pub mod cli;

use clap_complete::{generate, Shell};
use pylot::{activate, check, create, delete, install, list, uninstall, update};
use std::{io, str::FromStr};

use clap::{CommandFactory, Parser};
use cli::cmds::{Cli, Commands};
use shared::{logger, settings};

use crate::cli::cmds::{UvCommands, VenvCommands};

#[tokio::main]
async fn main() {
    settings::Settings::init().await;
    logger::initialize_logger(log::LevelFilter::Trace);
    let args = Cli::parse();

    match args.commands {
        Some(Commands::Complete { shell }) => {
            let shell = shell
                .as_deref()
                .and_then(|s| Shell::from_str(s).ok())
                .unwrap_or(Shell::Bash);
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "pylot", &mut io::stdout());
        }
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
            VenvCommands::Activate { name_pos, name } => activate(name_pos, name).await,
            VenvCommands::Create {
                name_pos,
                name,
                python_version,
                packages,
                requirements,
                default,
            } => {
                match create(
                    name_pos,
                    name,
                    python_version,
                    packages,
                    requirements,
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
                delete(io::stdin(), io::stdin(), name_pos, name).await
            }
            VenvCommands::List => list().await,
        },

        None => {
            log::error!("No command provided");
        }
    }
}
