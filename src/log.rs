use std::sync::OnceLock;

use env_logger::Env;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;

pub struct Logger {
    multi: MultiProgress,
}

impl Logger {
    pub fn init() -> Self {
        let logger =
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).build();
        let level = logger.filter();
        let multi = MultiProgress::new();

        LogWrapper::new(multi.clone(), logger).try_init().unwrap();
        log::set_max_level(level);

        Self { multi }
    }

    pub fn multi(&self) -> &MultiProgress {
        &self.multi
    }
}

pub fn logger() -> &'static Logger {
    static LOGGER: OnceLock<Logger> = OnceLock::new();
    LOGGER.get_or_init(|| Logger::init())
}
