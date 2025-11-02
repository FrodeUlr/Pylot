use super::styles;
use clap::{Parser, Subcommand};
use styles::custom_styles;

#[derive(Debug, Parser)]
#[command(
    version,
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true,
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
        visible_alias = "u",
        about = "Commands for managing UV",
        long_about = "This command group contains commands for managing Astral UV"
    )]
    Uv {
        #[command(subcommand)]
        command: UvCommands,
    },
    #[command(
        visible_alias = "v",
        about = "Commands for managing virtual environments",
        long_about = "This command group contains commands for managing Python virtual environments"
    )]
    Venv {
        #[command(subcommand)]
        command: VenvCommands,
    },
    #[command(
        visible_alias = "c",
        about = "Generate shell completion script",
        long_about = "Generates shell completion script for supported shells (bash, zsh, fish, powershell and elvish).\n\n\
            To enable completion, run:\n\
            pylot complete <shell> > ~/.pylot-completion.<shell>\n\
            Then add 'source ~/.pylot-completion.<shell>' to your shell config file (e.g., .bashrc, .zshrc, or .config/fish/config.fish).\n\
            Reload your shell to activate completions."
    )]
    Complete {
        #[arg(help = "Shell type", value_parser = ["bash", "zsh", "fish", "powershell", "elvish"], default_value = "bash")]
        shell: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum UvCommands {
    #[command(
        visible_alias = "i",
        about = "Install Astral UV",
        long_about = "This command installs Astral UV"
    )]
    Install,

    #[command(
        visible_alias = "up",
        about = "Update Astral UV",
        long_about = "This command updates Astral UV"
    )]
    Update,

    #[command(
        visible_alias = "u",
        about = "Uninstall Astral UV",
        long_about = "This command uninstalls Astral UV"
    )]
    Uninstall,

    #[command(
        about = "Check Astral UV",
        long_about = "This command checks if Astral UV is installed"
    )]
    Check,
}

#[derive(Subcommand, Debug)]
pub enum VenvCommands {
    #[command(
        visible_alias = "c",
        about = "Create a new python virtual environment",
        long_about = "This command creates a new python virtual environment"
    )]
    Create {
        #[arg(short, long, help = "Name of the virtual environment")]
        name: Option<String>,
        #[arg(
            short = 'v',
            visible_alias = "pv",
            long,
            help = "Python version to use",
            default_value = "3.10"
        )]
        python_version: String,
        #[arg(
            short = 'p',
            visible_alias = "pkg",
            long,
            help = "Packages to install",
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
        visible_aliases = ["d", "del"],
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
        visible_aliases = ["l", "ls"],
        about = "List all python virtual environments",
        long_about = "This command lists all python virtual environments"
    )]
    List,
    #[command(
        visible_alias = "a",
        about = "Activate a python virtual environment",
        long_about = "This command activates a python virtual environment in its own shell"
    )]
    Activate {
        #[arg(short, long, help = "Name of the virtual environment")]
        name: Option<String>,
        #[arg(index = 1, help = "Name of the virtual environment")]
        name_pos: Option<String>,
    },
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
