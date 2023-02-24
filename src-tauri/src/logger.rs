use log::SetLoggerError;
use log4rs::{
    self,
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::{json::JsonEncoder, pattern::PatternEncoder},
};

pub static LOG_FILES_PATH: &str = "C:/ProgramData/Tauri/EH File Version Monitor/logs/log-foo.log";

pub fn init() -> Result<(), SetLoggerError> {
    let level = log::LevelFilter::Info;

    // logging to console
    let stdout: ConsoleAppender = ConsoleAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new(
            "{h([{l}]):<10.15} {('{f}' line {L} -):<50.70} {m} {n}",
        )))
        .build();

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(JsonEncoder::new()))
        .build(LOG_FILES_PATH)
        .unwrap();

    let log_config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(level),
        )
        .unwrap();
    log4rs::init_config(log_config)?;

    Ok(())
}
