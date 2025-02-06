mod cfg;
mod cmd;
mod interface;

use cfg::settings;
use clap::Parser;
use cmd::manage;
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
            clean,
        }) => {
            let venv = venvmgr::Venv::new(name, python_version, clean);
            venv.create().await;
        }
        Some(Commands::Delete { name }) => {
            let venv = venvmgr::Venv::new(name, "".to_string(), false);
            venv.delete().await;
        }
        Some(Commands::List) => {
            venvmgr::Venv::list().await;
        }
        None => {
            println!("No command provided");
        }
    }
}
