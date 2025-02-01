mod cmd;
mod interface;

use clap::Arg;
use clap::builder::styling::{ Styles, AnsiColor };
use cmd::manage_uv::install_uv;

#[tokio::main]
async fn main(){
    //let args = Args::parse();

    let styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Green.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Green.on_default());

    let matches = clap::Command::new("pythonmanager")
        .about("A simple CLI to manage Python versions")
        .version("0.1.0")
        .styles(styles)
        .subcommand(
            clap::Command::new("install")
                .about("Install Astral UV")
                .arg(
                    Arg::new("version")
                        .short('v')
                        .long("version")
                        .help("The Python version to install")
                        .required(false)
                )
        )
        .subcommand(
            clap::Command::new("check")
                .about("Check if Astral UV is installed")
        )
        .get_matches();

    match matches.subcommand() {
        Some(("install", _)) => {
            install_uv(true).await;
        },
        Some(("check", _)) => {
            println!("Checking if Astral UV is installed");
        },
        _ => {
            println!("No command provided");
        }
    }
    //match args.commands {
    //    Some(Commands::Install { force }) => {
    //        install_uv(force);
    //    },
    //    Some(Commands::Check) => {
    //        println!("Checking if Astral UV is installed");
    //    },
    //    None => {
    //        println!("No command provided");
    //    }
    //}
}
