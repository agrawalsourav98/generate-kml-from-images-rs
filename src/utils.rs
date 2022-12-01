use std::{fs, path::PathBuf};

use log::{debug, warn};

static IMAGE_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "tiff", "tif"];

fn check_if_path_is_image(path: &PathBuf) -> bool {
    let ext = path.as_path().extension();
    if ext.is_none() {
        return false;
    }
    let ext = ext.unwrap().to_str();
    return IMAGE_EXTENSIONS.contains(&ext.unwrap().to_lowercase().as_str());
}

pub fn get_images_from_paths(image_paths: &Vec<String>) -> Vec<String> {
    let mut images: Vec<String> = Vec::new();
    for image_path in image_paths.iter() {
        let cpath = fs::canonicalize(&image_path);
        if cpath.is_err() {
            warn!("Skipping invalid file path {} provided", &image_path);
            continue;
        }
        let path = cpath.unwrap();
        if path.as_path().is_dir() {
            let paths = fs::read_dir(path.as_path()).unwrap();
            for image_p in paths {
                let p = image_p.unwrap().path();
                if check_if_path_is_image(&p) {
                    debug!("Found valid image file {:?}", p.display());
                    images.push(p.into_os_string().into_string().unwrap());
                } else {
                    debug!("Skipping non-image file {:?}", p.display())
                }
            }
        } else {
            images.push(path.into_os_string().into_string().unwrap())
        }
    }

    return images;
}
