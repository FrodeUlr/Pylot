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
