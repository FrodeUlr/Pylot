mod cmd;
mod interface;

use clap::Parser;
use cmd::manage::{ install, check, uninstall };
use interface::cli::{ Cli, Commands };

#[tokio::main]
async fn main(){
    let args = Cli::parse();

    match args.commands {
        Some(Commands::Install ) => {
            install().await;
        },
        Some(Commands::Uninstall) => {
            uninstall().await;
        }
        Some(Commands::Check) => {
            check().await;
        },
        None => {
            println!("No command provided");
        }
    }
}
