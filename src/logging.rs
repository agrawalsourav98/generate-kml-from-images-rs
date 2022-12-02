use simplelog::{
    format_description, ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, LevelPadding,
    TermLogger, TerminalMode, ThreadLogMode, WriteLogger,
};

use std::fs::File;

use crate::{
    structures::{Error, LogLevel},
    utils::create_parent_directory,
};

fn get_level_filter(level: LogLevel) -> LevelFilter {
    match level {
        LogLevel::Off => LevelFilter::Off,
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warning => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    }
}

pub fn setup_logger(level: LogLevel, file_level: LogLevel, logfile: &str) -> Result<(), Error> {
    create_parent_directory(logfile)?;
    let config = ConfigBuilder::new()
        .set_thread_level(LevelFilter::Debug)
        .set_target_level(LevelFilter::Error)
        .set_location_level(LevelFilter::Off)
        .set_level_padding(LevelPadding::Right)
        .set_thread_mode(ThreadLogMode::Names)
        .set_time_format_custom(format_description!("[year]-[month]-[day]T[hour repr:24]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory]:[offset_minute]"))
        .set_time_offset_to_local().unwrap()
        .build();
    CombinedLogger::init(vec![
        TermLogger::new(
            get_level_filter(level),
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            get_level_filter(file_level),
            config,
            File::create(logfile).unwrap(),
        ),
    ])?;
    Ok(())
}
