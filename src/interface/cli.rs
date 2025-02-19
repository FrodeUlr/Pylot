use super::cli_styles;
use clap::{Parser, Subcommand};
use cli_styles::custom_styles;

#[derive(Debug, Parser)]
#[command(
    version,
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true
)]
#[command(
    name = "Python Manager",
    version = "0.1.0",
    author = "Fulrix",
    about = "A simple CLI to manage Python virtual enviroonments using Astral UV",
    styles = custom_styles()
)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(
        about = "Install Astral UV",
        long_about = "This command installs Astral UV"
    )]
    Install,

    #[command(
        about = "Check Astral UV",
        long_about = "This command checks if Astral UV is installed"
    )]
    Check,

    #[command(
        about = "Uninstall Astral UV",
        long_about = "This command uninstalls Astral UV"
    )]
    Uninstall,

    #[command(
        about = "Create a new python virtual environment",
        long_about = "This command creates a new python virtual environment"
    )]
    Create {
        #[arg(short, long, help = "Name of the virtual environment")]
        name: String,
        #[arg(short, long, help = "Python version to use", default_value = "3.10")]
        python_version: String,
        #[arg(
            short = 'k',
            long,
            help = "Packages to install",
            default_value = "[]",
            num_args = 1..
        )]
        packages: Vec<String>,
    },
    #[command(
        about = "Delete a python virtual environment",
        long_about = "This command deletes a python virtual environment"
    )]
    Delete {
        #[arg(short, long, help = "Name of the virtual environment")]
        name: String,
    },
    #[command(
        about = "List all python virtual environments",
        long_about = "This command lists all python virtual environments"
    )]
    List,
    #[command(
        about = "Activate a python virtual environment",
        long_about = "This command activates a python virtual environment in its own shell"
    )]
    Activate {
        #[arg(short, long, help = "Name of the virtual environment")]
        name: String,
    },
}
