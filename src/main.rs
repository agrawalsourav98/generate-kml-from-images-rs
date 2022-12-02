use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use structures::Error;

use crate::generate_kml::generate_kml_from_images;
use crate::logging::setup_logger;
use crate::structures::LogLevel;
use crate::utils::create_parent_directory;
use log::{debug, error, info};

mod generate_kml;
mod logging;
mod structures;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The log level of the program
    #[arg(short,long,value_enum, default_value_t = LogLevel::Info)]
    log_level: structures::LogLevel,
    /// The log level of the program
    #[arg(long,value_enum, default_value_t = LogLevel::Debug)]
    file_log_level: structures::LogLevel,
    /// The file to write log to
    #[arg(long, default_value_t = ("logs/generate_kml.log").to_string())]
    log_file: String,
    /// The output kml file
    #[arg(short, long, default_value_t = ("output.kml").to_string())]
    /// The output kml location
    kml_file: String,
    /// The list of images locations, can be a combination of files and directories
    images: Vec<String>,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match create_parent_directory(&cli.log_file) {
        Err(e) => error!("Failed to create log directory: {}", e),
        Ok(parent) => debug!("Log directory {} created successfully", parent),
    }

    setup_logger(cli.log_level, cli.file_log_level, &cli.log_file)?;

    if cli.images.is_empty() {
        let mut cmd = Cli::command();
        cmd.error(
            ErrorKind::MissingRequiredArgument,
            "You must provide multiple images path or a directory of images or both",
        )
        .exit()
    }

    debug!("Provided images arg: {:?}", cli.images);

    let images = utils::get_images_from_paths(&cli.images);

    info!("Found {} images", images.len());

    match generate_kml_from_images(&images, &cli.kml_file) {
        Err(e) => error!("Error while creating kml: {}", e),
        Ok(()) => info!("KML file {} created successfully", cli.kml_file),
    }
    Ok(())
}
