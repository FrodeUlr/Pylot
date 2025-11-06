use std::sync::Once;

use log::LevelFilter;
use shared::logger::initialize_logger;

static INIT: Once = Once::new();

pub fn setup_logger() {
    INIT.call_once(|| {
        initialize_logger(LevelFilter::Trace);
    });
}
