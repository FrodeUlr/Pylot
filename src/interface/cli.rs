use clap::{  Parser, Subcommand };
use cli_styles::custom_styles;

use super::cli_styles;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = true)]
#[command(
    name = "pythonmanager",
    version = "0.1.0",
    author = "Fulrix",
    about = "A simple CLI to manage Python virtual enviroonments using Astral UV",
    styles = custom_styles()
)]
pub struct Args {
    #[command(subcommand)]
    pub commands: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Install Astral UV", long_about = "This command installs Astral UV")]
    Install,

    #[command(about = "Check Astral UV", long_about = "This command checks if Astral UV is installed")]
    Check,
}

