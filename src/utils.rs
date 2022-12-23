use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use exif::{In, Reader, Tag, Value};
use log::{error, trace, warn};

use crate::structures::{Error, GPSInformation, GPSInformationField};

static IMAGE_EXTENSIONS: [&str; 9] = [
    "png", "jpg", "jpeg", "tiff", "tif", "webp", "heif", "heic", "avif",
];

static TAG_LIST: [Tag; 6] = [
    Tag::GPSAltitude,
    Tag::GPSAltitudeRef,
    Tag::GPSLatitude,
    Tag::GPSLatitudeRef,
    Tag::GPSLongitude,
    Tag::GPSLongitudeRef,
];

#[macro_export]
macro_rules! file_name {
    ($input:expr) => {
        Path::new($input).file_name().unwrap().to_str().unwrap()
    };
}

#[macro_export]
macro_rules! path_to_string {
    ($input:expr) => {
        $input.as_path().to_str().unwrap()
    };
}

pub fn create_parent_directory(path: &str) -> Result<String, Error> {
    let parent = Path::new(path).parent();
    if let Some(parent) = parent {
        std::fs::create_dir_all(parent)?;
        return Ok(parent.to_str().unwrap().to_string());
    }
    Ok("No parent found for the given path".to_string())
}

pub fn check_if_path_is_image(path: &Path) -> bool {
    let ext = path.extension();
    if ext.is_none() {
        return false;
    }
    let ext = ext.unwrap().to_str();
    return IMAGE_EXTENSIONS.contains(&ext.unwrap().to_lowercase().as_str());
}

pub fn get_images_from_paths(image_paths: &[String]) -> Vec<String> {
    let mut images: Vec<String> = Vec::new();
    for image_path in image_paths.iter() {
        let absolute = fs::canonicalize(image_path);
        if absolute.is_err() {
            warn!("Skipping invalid file path {} provided", &image_path);
            continue;
        }
        let path = absolute.unwrap();
        if path.as_path().is_dir() {
            trace!("Adding images from directory {}", path_to_string!(&path));
            let paths = fs::read_dir(path.as_path()).unwrap();
            for image_p in paths {
                let p = image_p.unwrap().path();
                if check_if_path_is_image(&p) {
                    trace!("Found valid image file {}", file_name!(&p));
                    images.push(path_to_string!(p).to_string());
                } else {
                    trace!("Skipping non-image file {}", file_name!(&p))
                }
            }
        } else {
            trace!("Adding image {}", file_name!(&path));
            images.push(path_to_string!(path).to_string());
        }
    }

    images.sort();

    images
}

pub fn get_gps_information(image: &str) -> Result<GPSInformation, Error> {
    trace!("Fetching GPS Information for image {}", file_name!(image));
    let mut gps_info = GPSInformation::new();
    let file = File::open(image)?;
    let exif = Reader::new().read_from_container(&mut BufReader::new(&file))?;

    for (idx, &tag) in TAG_LIST.iter().enumerate() {
        if let Some(field) = exif.get_field(tag, In::PRIMARY) {
            trace!("Field value for tag {}:{:?}", tag, field.value);
            match field.value {
                Value::Rational(ref vec) if !vec.is_empty() => {
                    if vec.len() == 1 {
                        gps_info.set_index(&idx, GPSInformationField::Float(vec[0].to_f64()))
                    } else {
                        gps_info.set_index(
                            &idx,
                            GPSInformationField::Float(
                                vec[0].to_f64()
                                    + vec[1].to_f64() / 60_f64
                                    + vec[2].to_f64() / 3600_f64,
                            ),
                        )
                    }
                }
                Value::Byte(ref byte) if !byte.is_empty() => {
                    gps_info.set_index(&idx, GPSInformationField::Int(byte[0]))
                }
                Value::Ascii(ref vec) if !vec.is_empty() => {
                    gps_info.set_index(&idx, GPSInformationField::Char(vec[0][0] as char))
                }
                _ => error!(
                    "Invalid value encountered for tag {}:{:?}",
                    tag, field.value
                ),
            }
        }
    }
    Ok(gps_info)
}
