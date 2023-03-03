use log::SetLoggerError;
use log4rs::{
    self,
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::compound::{
                roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Root},
    encode::{json::JsonEncoder, pattern::PatternEncoder},
};

pub static LOG_FILES_PATH: &str = "C:/ProgramData/Tauri/EH File Version Monitor/logs";
pub static LOG_FILE_NAME: &str = "application-logs";

pub fn init() -> Result<(), SetLoggerError> {
    // TODO: make log level changeable and take it from the db - user can change it in the settings page
    let level = log::LevelFilter::Info;

    // logging to console
    let stdout: ConsoleAppender = ConsoleAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new(
            "{h([{l}]):<10.15} {('{f}' line {L} -):<50.70} {m} {n}",
        )))
        .build();

    // Logging to log file (rolling log file).
    let window_size = 5; // files to keep
    let fixed_window_roller = FixedWindowRoller::builder()
        .build(
            &format!("{LOG_FILES_PATH}/{LOG_FILE_NAME}.{{}}.log"),
            window_size,
        )
        .unwrap();

    let size_limit = 20 * 1024 * 1000; // 20MB as max log file size to roll
    let size_trigger = SizeTrigger::new(size_limit);
    let compound_policy =
        CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));

    let log_config = log4rs::config::Config::builder()
        .appender(
            Appender::builder().build(
                "rollinglogfile",
                Box::new(
                    RollingFileAppender::builder()
                        .encoder(Box::new(JsonEncoder::new()))
                        .build(
                            &format!("{LOG_FILES_PATH}/{LOG_FILE_NAME}.log"),
                            Box::new(compound_policy),
                        )
                        .unwrap(),
                ),
            ),
        )
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("rollinglogfile")
                .build(level),
        )
        .unwrap();
    log4rs::init_config(log_config)?;

    Ok(())
}
