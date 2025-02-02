mod cmd;
mod interface;

use clap::Parser;
use cmd::manage_uv::{ install_uv, check_uv };
use interface::cli::{ Cli, Commands };

#[tokio::main]
async fn main(){
    let args = Cli::parse();

    match args.commands {
        Some(Commands::Install) => {
            install_uv().await;
        },
        Some(Commands::Check) => {
            check_uv().await;
        },
        None => {
            println!("No command provided");
        }
    }
}
