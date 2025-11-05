#[cfg(test)]
mod tests {
    use crate::cli::cmds::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
