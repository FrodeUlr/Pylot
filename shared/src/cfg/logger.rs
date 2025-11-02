use env_logger::Builder;
use log::Level;

pub fn initialize_logger(log_level: log::LevelFilter) {
    Builder::new()
        .filter_level(log_level)
        .format(|buf, record| {
            use std::io::Write;
            let level_color = match record.level() {
                Level::Error => "\x1b[31m", // Red
                Level::Warn => "\x1b[33m",  // Yellow
                Level::Info => "\x1b[32m",  // Green
                Level::Debug => "\x1b[36m", // Cyan
                Level::Trace => "\x1b[35m", // Magenta
            };
            writeln!(buf, "{:5}{}", level_color, record.args())
        })
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, error, info, trace, warn, LevelFilter};
    use std::sync::Once;

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
