#[cfg(test)]
mod tests {
    use log::{debug, error, info, trace, warn, LevelFilter};
    use std::sync::Once;

    use shared::logger::initialize_logger;

    static INIT: Once = Once::new();

    fn setup_logger() {
        INIT.call_once(|| {
            initialize_logger(LevelFilter::Trace);
        });
    }

    #[test]
    fn test_logger_output() {
        setup_logger();

        let msg = "test message";

        error!("{}", msg);
        warn!("{}", msg);
        info!("{}", msg);
        debug!("{}", msg);
        trace!("{}", msg);
    }
}
