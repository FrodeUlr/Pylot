use super::styles;
use clap::{Parser, Subcommand};
use styles::custom_styles;

/// Command Line Interface for Pylot
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

/// Pylot top level commands
///
/// # Usage
/// * `pylot uv` - UV management commands
/// * `pylot venv` - Virtual environment management commands
/// * `pylot complete` - Shell completion script generation
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// UV management commands
    ///
    /// # Usage
    /// * `pylot uv` - Work with Astral UV
    /// * `pylot u` - Work with Astral UV (alias)
    #[command(
        visible_alias = "u",
        about = "Commands for managing UV",
        long_about = "This command group contains commands for managing Astral UV"
    )]
    Uv {
        #[command(subcommand)]
        command: UvCommands,
    },
    /// Virtual environment management commands
    ///
    /// # Usage
    /// * `pylot venv` - Manage Python virtual environments
    /// * `pylot v` - Manage Python virtual environments (alias)
    #[command(
        visible_alias = "v",
        about = "Commands for managing virtual environments",
        long_about = "This command group contains commands for managing Python virtual environments"
    )]
    Venv {
        #[command(subcommand)]
        command: VenvCommands,
    },
    /// Shell completion script generation
    ///
    /// # Usage
    /// * `pylot complete bash` - Generate bash completion script
    /// * `pylot c zsh` - Generate zsh completion script (alias)
    /// * `pylot complete` - Generate bash completion script (default)
    /// * `pylot c powershell` - Generate powershell completion script (alias)
    /// * `pylot complete fish` - Generate fish completion script
    /// * `pylot complete elvish` - Generate elvish completion script
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

/// UV management commands
///
/// # Usage
/// * `pylot uv` - UV management commands
/// * `pylot u` - UV management commands (alias)
#[derive(Subcommand, Debug)]
pub enum UvCommands {
    /// Install Astral UV
    ///
    /// # Usage
    /// * `pylot uv install` - Install Astral UV
    /// * `pylot u i` - Install Astral UV (alias)
    #[command(
        visible_alias = "i",
        about = "Install Astral UV",
        long_about = "This command installs Astral UV"
    )]
    Install,
    /// Update Astral UV
    ///
    /// # Usage
    /// * `pylot uv update` - Update Astral UV
    /// * `pylot u up` - Update Astral UV (alias)
    #[command(
        visible_alias = "up",
        about = "Update Astral UV",
        long_about = "This command updates Astral UV"
    )]
    Update,
    /// Uninstall Astral UV
    ///
    /// # Usage
    /// * `pylot uv uninstall` - Uninstall Astral UV
    /// * `pylot u u` - Uninstall Astral UV (alias)
    #[command(
        visible_alias = "u",
        about = "Uninstall Astral UV",
        long_about = "This command uninstalls Astral UV"
    )]
    Uninstall,
    /// Check if Astral UV is installed
    ///
    /// # Usage
    /// * `pylot uv check` - Check if Astral UV is installed
    /// * `pylot u c` - Check if Astral UV is installed (alias)
    #[command(
        visible_alias = "c",
        about = "Check Astral UV",
        long_about = "This command checks if Astral UV is installed"
    )]
    Check,
}

/// Virtual environment management commands
///
/// # Usage
/// * `pylot venv` - Virtual environment management commands
#[derive(Subcommand, Debug)]
pub enum VenvCommands {
    /// Create a new virtual environment
    ///
    /// # Usage
    /// * `pylot venv create myenv -v 3.9 -p numpy pandas` - Create a virtual environment named `myenv` with Python 3.9 and install `numpy` and `pandas`
    /// * `pylot v c myenv --requirements requirements.txt` - Create a virtual environment named `myenv` and install packages from `requirements.txt` (alias)
    /// * `pylot v c -n myenv -v 3.8 -d -p flask django` - Create a virtual environment named `myenv` with Python 3.8 and install default packages along with `flask` and `django` (alias)
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
    /// Delete a virtual environment
    ///
    /// # Usage
    /// * `pylot venv delete myenv` - Delete the virtual environment named `myenv`
    /// * `pylot v d -n myenv` - Delete the virtual environment named `myenv` (alias)
    /// * `pylot v d` - List virtual environments and prompt to select one to delete (alias)
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
    /// List all virtual environments
    ///
    /// # Usage
    /// * `pylot venv list` - List all python virtual environments
    /// * `pylot v l` - List all python virtual environments (alias)
    #[command(
        visible_aliases = ["l", "ls"],
        about = "List all python virtual environments",
        long_about = "This command lists all python virtual environments"
    )]
    List,
    /// Activate a virtual environment
    ///
    /// # Usage
    /// * `pylot venv activate myenv` - Activate the virtual environment named `myenv`
    /// * `pylot v a -n myenv` - Activate the virtual environment named `myenv` (alias)
    /// * `pylot v a` - List virtual environments and prompt to select one to activate (alias)
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
