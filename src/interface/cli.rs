use clap::{ Parser, Subcommand };

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub commands: Option<Commands>,

    #[arg(short, long, default_value_t = 1)]
    pub count: u8,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Install Astral UV", long_about = "This command installs Astral UV")]
    Install {
        #[arg(short, long, help = "Force flag (default: false)", default_value = "false")]
        force: bool,
    },

    #[command(about = "Check Astral UV", long_about = "This command checks if Astral UV is installed")]
    Check,
}

