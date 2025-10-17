use super::styles;
use clap::{Parser, Subcommand};
use styles::custom_styles;

#[derive(Debug, Parser)]
#[command(
    version,
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true
)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
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
        name = "install-uv",
        about = "Install Astral UV",
        long_about = "This command installs Astral UV"
    )]
    Install {
        #[arg(
            short = 'u',
            long,
            help = "Check for updates and install if available",
            default_value = "false"
        )]
        update: bool,
    },

    #[command(
        about = "Check Astral UV",
        long_about = "This command checks if Astral UV is installed"
    )]
    Check,

    #[command(
        name = "uninstall-uv",
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
        name: Option<String>,
        #[arg(
            short = 'v',
            alias = "pv",
            long,
            help = "Python version to use(alias --pv)",
            default_value = "3.10"
        )]
        python_version: String,
        #[arg(
            short = 'p',
            alias = "pkg",
            long,
            help = "Packages to install(alias --pkg)",
            num_args = 1..
        )]
        packages: Vec<String>,
        #[arg(
            short = 'r',
            long,
            help = "Requirements file to install packages from",
            default_value = ""
        )]
        requirements: String,
        #[arg(index = 1, help = "Name of the virtual environment")]
        name_pos: Option<String>,
        #[arg(short, long, help = "Use default packages")]
        default: bool,
    },
    #[command(
        about = "Delete a python virtual environment",
        long_about = "This command deletes a python virtual environment"
    )]
    Delete {
        #[arg(short, long, help = "Name of the virtual environment")]
        name: Option<String>,
        #[arg(index = 1, help = "Name of the virtual environment")]
        name_pos: Option<String>,
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
        name: Option<String>,
        #[arg(index = 1, help = "Name of the virtual environment")]
        name_pos: Option<String>,
    },
    #[command(
        about = "Launch TUI for managing python virtual environments",
        long_about = "This command launches a TUI for managing python virtual environments"
    )]
    Tui,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
