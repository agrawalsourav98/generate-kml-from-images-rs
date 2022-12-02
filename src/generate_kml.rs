use log::{debug, info};
use std::{fs::File, io::Write, path::Path};

use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Writer,
};

use crate::{
    file_name,
    utils::{create_parent_directory, get_gps_information},
};

use super::structures::Error;

fn write_element(writer: &mut Writer<Vec<u8>>, tag: &str, content: &str) -> Result<(), Error> {
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    writer.write_event(Event::Text(BytesText::new(content)))?;
    writer.write_event(Event::End(BytesEnd::new(tag)))?;

    Ok(())
}

fn write_kml(content: &Vec<u8>, filename: &str) -> Result<(), Error> {
    create_parent_directory(filename)?;
    let mut f = File::create(filename)?;
    f.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    f.write_all(content)?;
    debug!(
        "Successfully written {} bytes to file {}",
        content.len() + 38,
        filename
    );
    Ok(())
}

pub fn generate_kml_from_images(images: &[String], filename: &str) -> Result<(), Error> {
    let buf = Vec::new();
    let mut writer = Writer::new_with_indent(buf, b' ', 4);

    let mut root = BytesStart::new("kml");
    root.push_attribute(("xmlns", "http://www.opengis.net/kml/2.2"));
    writer.write_event(Event::Start(root))?;
    writer.write_event(Event::Start(BytesStart::new("Folder")))?;
    write_element(
        &mut writer,
        "name",
        Path::new(filename).file_stem().unwrap().to_str().unwrap(),
    )?;

    let mut count: [usize; 3] = [0, 0, 0];

    for (idx, image) in images.iter().enumerate() {
        let gps_information = match get_gps_information(image) {
            Err(e) => {
                count[0] += 1;
                info!("Failed to get gps information for {}: {}", image, e);
                continue;
            }
            Ok(value) => value,
        };
        if !gps_information.is_valid() {
            count[1] += 1;
            info!("No GPS Information found in image {}", image);
            continue;
        }
        let altitude = gps_information.get_param("alt");
        let longitude = gps_information.get_param("lon");
        let latitude = gps_information.get_param("lat");
        let description = format!(
            "&lt;p&gt;Filename: {}&lt;/i&gt;
            &lt;p&gt;Longitude: {}&lt;/i&gt;
            &lt;p&gt;Lotitude: {}&lt;/i&gt;
            &lt;p&gt;Altitude: {}&lt;/i&gt;",
            file_name!(image),
            longitude,
            latitude,
            altitude
        );
        writer.write_event(Event::Start(BytesStart::new("Placemark")))?;
        write_element(&mut writer, "name", &(idx + 1).to_string())?;
        write_element(&mut writer, "description", &description)?;
        writer.write_event(Event::Start(BytesStart::new("Point")))?;
        write_element(
            &mut writer,
            "coordinates",
            &format!("{},{},{}", longitude, latitude, altitude),
        )?;
        writer.write_event(Event::End(BytesEnd::new("Point")))?;
        writer.write_event(Event::End(BytesEnd::new("Placemark")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("Folder")))?;
    writer.write_event(Event::End(BytesEnd::new("kml")))?;

    let result = writer.into_inner();
    write_kml(&result, filename)?;
    count[2] += 1;
    debug!("Successfully generated kml for {} images,skipped {} images due to invalid_exif and {} images due to no GPS information",count[0],count[1],count[2]);
    Ok(())
}
