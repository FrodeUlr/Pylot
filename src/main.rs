mod cfg;
mod cmd;
mod core;

use cfg::settings;
use clap::Parser;
use cmd::manage;
use cmd::utils;
use cmd::venvmgr;
use colored::Colorize;
use core::cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    settings::Settings::init().await;
    let args = Cli::parse();

    match args.commands {
        Some(Commands::Install) => {
            manage::install().await;
        }
        Some(Commands::Uninstall) => {
            manage::uninstall().await;
        }
        Some(Commands::Check) => {
            println!("{}", "Checking if Astral UV is installed and configured...".cyan());
            let installed = manage::check().await;
            if installed {
                println!("{}", "Astral UV is installed".green());
            } else {
                println!("{}", "Astral UV was not found".red());
            }
        }
        Some(Commands::Create {
            name_pos,
            name,
            python_version,
            packages,
            default,
        }) => {
            let name = name.or(name_pos).unwrap_or_else(|| {
                utils::exit_with_error("Error, please provide a name for the virtual environment")
            });
            let mut packages = packages;
            if default {
                let default_pkgs = settings::Settings::get_settings().default_pkgs.clone();
                packages.extend(default_pkgs);
            }
            let venv = venvmgr::Venv::new(name, python_version, packages);
            venv.create().await;
        }
        Some(Commands::Delete { name_pos, name }) => {
            let name  = name.or(name_pos).unwrap_or_else(|| {
                utils::exit_with_error("Error, please provide a envioronment name")
            });
            let venv = venvmgr::Venv::new(name, "".to_string(), vec![]);
            venv.delete().await;
        }
        Some(Commands::List) => {
            venvmgr::Venv::list().await;
        }
        Some(Commands::Activate { name_pos, name }) => {
            let name = name.or(name_pos).unwrap_or_else(|| {
                utils::exit_with_error("Error, please provide a envioronment name")
            });
            let venv = venvmgr::Venv::new(name, "".to_string(), vec![]);
            venv.activate().await;
        }
        None => {
            println!("No command provided");
        }
    }
}
