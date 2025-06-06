mod cfg;
mod cmd;
mod core;
mod utility;

use cfg::settings;
use clap::Parser;
use cmd::{manage, utils, venvmgr};
use colored::Colorize;
use core::cli::{Cli, Commands};
use std::io;
use utility::util;

#[tokio::main]
async fn main() {
    settings::Settings::init().await;
    let args = Cli::parse();

    match args.commands {
        Some(Commands::Install { update }) => {
            if !update && manage::check().await {
                println!(
                    "{}",
                    "Astral UV is already installed, skipping installation.\nUse --update (-u) flag to check for update".yellow()
                );
                return;
            }
            if let Err(e) = manage::install(io::stdin()).await {
                eprintln!("{}", format!("Error installing Astral UV: {}", e).red());
            }
        }

        Some(Commands::Uninstall) => {
            if let Err(e) = manage::uninstall(io::stdin()).await {
                eprintln!("{}", format!("Error uninstalling Astral UV: {}", e).red());
            }
        }

        Some(Commands::Check) => {
            println!(
                "{}",
                "Checking if Astral UV is installed and configured...".cyan()
            );
            if manage::check().await {
                println!("{}", "Astral UV is installed".green());
                return;
            }
            println!("{}", "Astral UV was not found".red());
        }

        Some(Commands::Create {
            name_pos,
            name,
            python_version,
            packages,
            default,
        }) => {
            let name = match name.or(name_pos) {
                Some(n) => n,
                None => {
                    utils::exit_with_error("Missing name for the environment.");
                }
            };
            let venv = venvmgr::Venv::new(name, python_version, packages, default);
            if let Err(e) = venv.create().await {
                eprintln!(
                    "{}",
                    format!("Error creating virtual environment: {}", e).red()
                );
            }
        }

        Some(Commands::Delete { name_pos, name }) => {
            let venv = util::find_venv(name_pos, name, "delete".to_string()).await;
            match venv {
                Some(v) => v.delete().await,
                None => return,
            }
        }

        Some(Commands::List) => {
            venvmgr::Venv::list(Some(true)).await;
        }

        Some(Commands::Activate { name_pos, name }) => {
            let venv = util::find_venv(name_pos, name, "activate".to_string()).await;
            match venv {
                Some(v) => v.activate().await,
                None => return,
            }
        }
        None => {
            println!("No command provided");
        }
    }
}
