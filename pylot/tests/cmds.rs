#[cfg(test)]
mod tests {
    use pylot::cli::cmds::Cli;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
