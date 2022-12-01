use std::fs::File;
use std::path::Path;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

use crate::generate_kml::generate_kml_from_images;
use crate::structures::LogLevel;
//use env_logger::{Env, Target};
use log::{debug, error, info, LevelFilter};

pub mod generate_kml;
pub mod structures;
pub mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The log level of the program
    #[arg(short,long,value_enum, default_value_t = LogLevel::Info)]
    log_level: structures::LogLevel,
    #[arg(long, default_value_t = ("logs/generate_kml.log").to_string())]
    log_file: String,
    #[arg(short, long, default_value_t = ("output.kml").to_string())]
    /// The output kml location
    kml_file: String,
    /// The list of images
    images: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    // let env = Env::default()
    //     .filter_or("LOG_LEVEL", cli.log_level.to_string())
    //     .write_style_or("LOG_STYLE", "auto");

    // env_logger::Builder::from_env(env)
    //     .format_timestamp_millis()
    //     .target(Target::Pipe())
    //     .init();

    let level = match cli.log_level {
        LogLevel::Off => LevelFilter::Off,
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warning => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    };

    let parent = Path::new(&cli.log_file).parent();
    match std::fs::create_dir_all(parent.unwrap()) {
        Err(e) => error!("Failed to create log directory {:?} : {}", parent, e),
        Ok(()) => debug!("Log directory {:?} created successfully", parent),
    }

    CombinedLogger::init(vec![
        TermLogger::new(
            level,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(cli.log_file).unwrap(),
        ),
    ])
    .unwrap();

    // match WriteLogger::init(
    //     LevelFilter::Trace,
    //     Config::default(),
    //     File::create(cli.log_file).unwrap(),
    // ) {
    //     Err(e) => error!("Failed to create file logger: {}", e),
    //     Ok(()) => debug!("File logger created successfully"),
    // }

    if cli.images.is_empty() {
        let mut cmd = Cli::command();
        cmd.error(
            ErrorKind::MissingRequiredArgument,
            "You must provide multiple images path or a directory of images or both",
        )
        .exit()
    }

    info!("Provided images arg: {:?}", cli.images);

    let images = utils::get_images_from_paths(&cli.images);

    info!("Found {} images", images.len());

    let res = generate_kml_from_images(&images, &cli.kml_file);
    match res {
        Err(e) => error!("Error while creating kml: {}", e),
        Ok(()) => info!("KML file {} created successfully", cli.kml_file),
    }
}
