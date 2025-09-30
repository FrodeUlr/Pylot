mod cfg;
mod cli_core;
mod cmd;
mod utility;

use cfg::settings;
use clap::Parser;
use cli_core::cli::{Cli, Commands};

use crate::cli_core::run::{
    run_activate, run_check, run_create, run_delete, run_install, run_list, run_uninstall,
};

#[tokio::main]
async fn main() {
    settings::Settings::init().await;
    let args = Cli::parse();

    match args.commands {
        Some(Commands::Activate { name_pos, name }) => run_activate(name_pos, name).await,

        Some(Commands::Check) => run_check().await,

        Some(Commands::Create {
            name_pos,
            name,
            python_version,
            packages,
            requirements,
            default,
        }) => {
            run_create(
                name_pos,
                name,
                python_version,
                packages,
                requirements,
                default,
            )
            .await
        }

        Some(Commands::Delete { name_pos, name }) => run_delete(name_pos, name).await,

        Some(Commands::List) => run_list().await,

        Some(Commands::Install { update }) => run_install(update).await,

        Some(Commands::Uninstall) => run_uninstall().await,

        None => {
            println!("No command provided");
        }
    }
}
