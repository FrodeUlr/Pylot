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
    let settings = settings::Settings::get_settings();
    println!("{:?}", settings);
    println!("{:?}", settings.location);
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
        None => {
            println!("No command provided");
        }
    }
}
