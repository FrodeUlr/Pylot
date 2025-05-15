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
        Some(Commands::Install) => {
            manage::install(io::stdin()).await;
        }

        Some(Commands::Uninstall) => {
            manage::uninstall(io::stdin()).await;
        }

        Some(Commands::Check) => {
            println!(
                "{}",
                "Checking if Astral UV is installed and configured...".cyan()
            );
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
            let venv = venvmgr::Venv::new(name, python_version, packages, default);
            venv.create().await;
        }

        Some(Commands::Delete { name_pos, name }) => {
            let venv = if name.is_none() && name_pos.is_none() {
                let venvs = venvmgr::Venv::list(Some(false)).await;
                if venvs.is_empty() {
                    return;
                }
                for (i, venv) in venvs.iter().enumerate() {
                    println!("{}. {}", i + 1, venv);
                }
                println!("Please select a virtual environment to delete:");
                let index = util::get_index(venvs.len());
                venvmgr::Venv::new(venvs[index - 1].clone(), "".to_string(), vec![], false)
            } else {
                let name = name.or(name_pos).unwrap_or_else(|| {
                    utils::exit_with_error("Error, please provide an environment name")
                });
                venvmgr::Venv::new(name, "".to_string(), vec![], false)
            };
            venv.delete().await;
        }

        Some(Commands::List) => {
            venvmgr::Venv::list(Some(true)).await;
        }

        Some(Commands::Activate { name_pos, name }) => {
            let name = if name.is_none() && name_pos.is_none() {
                let venvs = venvmgr::Venv::list(Some(false)).await;
                if venvs.is_empty() {
                    return;
                }
                for (i, venv) in venvs.iter().enumerate() {
                    println!("{}. {}", i + 1, venv);
                }
                println!("Please select a virtual environment to activate:");
                {
                    let index = util::get_index(venvs.len());
                    venvs[index - 1].clone()
                }
            } else {
                name.or(name_pos).unwrap_or_else(|| {
                    utils::exit_with_error("Error, please provide a environment name")
                })
            };
            let venv = venvmgr::Venv::new(name, "".to_string(), vec![], false);
            venv.activate().await;
        }
        None => {
            println!("No command provided");
        }
    }
}
