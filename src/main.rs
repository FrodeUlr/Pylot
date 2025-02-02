mod cmd;
mod interface;

use clap::Parser;
use cmd::manage_uv::{ install_uv, check_uv };
use interface::cli::{Args, Commands};

#[tokio::main]
async fn main(){
    let args = Args::parse();

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

    let _ = Args::parse();
}
