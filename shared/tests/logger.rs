mod helpers;

#[cfg(test)]
mod tests {
    use log::{debug, error, info, trace, warn};

    use crate::helpers::setup_logger;

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
