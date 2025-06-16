use chrono::Local;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use colored::*;

struct LocalTimeLogger;

impl log::Log for LocalTimeLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S");
            let level = record.level();
            let message = record.args();
            
            let colored_level = match level {
                Level::Error => level.to_string().red(),
                Level::Warn => level.to_string().yellow(),
                Level::Info => level.to_string().cyan(),
                Level::Debug => level.to_string().purple(),
                Level::Trace => level.to_string().normal(),
            };
            
            println!("{} [{}] - {}", now, colored_level, message);
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(LocalTimeLogger))
        .map(|()| log::set_max_level(LevelFilter::Info))
}