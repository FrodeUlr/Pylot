mod cfg;
mod cmd;
mod interface;

use cfg::settings;
use clap::Parser;
use cmd::manage;
use cmd::utils;
use cmd::venvmgr;
use interface::cli::{Cli, Commands};

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
            manage::check().await;
        }
        Some(Commands::Create {
            name,
            python_version,
            packages,
            name_pos,
        }) => {
            let name = name.or(name_pos).unwrap_or_else(|| {
                utils::exit_with_error("Please provide a name for the virtual environment")
            });
            let venv = venvmgr::Venv::new(name, python_version, packages);
            venv.create().await;
        }
        Some(Commands::Delete { name }) => {
            let venv = venvmgr::Venv::new(name, "".to_string(), vec![]);
            venv.delete().await;
        }
        Some(Commands::List) => {
            venvmgr::Venv::list().await;
        }
        Some(Commands::Activate { name }) => {
            let venv = venvmgr::Venv::new(name, "".to_string(), vec![]);
            venv.activate().await;
        }
        None => {
            println!("No command provided");
        }
    }
}
