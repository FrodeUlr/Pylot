mod cmd;
mod interface;

use clap::Parser;
use interface::cli::{ Args, Commands };
use cmd::manage_uv::install_uv;

fn main(){
    let args = Args::parse();

    match args.commands {
        Some(Commands::Install { force }) => {
            install_uv(force);
        },
        Some(Commands::Check) => {
            println!("Checking if Astral UV is installed");
        },
        None => {
            println!("No command provided");
        }
    }
}
