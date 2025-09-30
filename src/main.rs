mod cfg;
mod cli;
mod core;
mod shell;
mod utility;

use cfg::settings;
use clap::Parser;
use cli::clicmd::{Cli, Commands};

use crate::cli::run;

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
