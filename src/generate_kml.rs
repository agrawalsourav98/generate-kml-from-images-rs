use log::{debug, info};
use std::{fs::File, io::Write, path::Path};

use quick_xml::{
    events::{BytesCData, BytesDecl, BytesEnd, BytesStart, BytesText, Event},
    Writer,
};

use crate::{
    file_name,
    utils::{create_parent_directory, get_gps_information},
};

use super::structures::Error;

enum ContentType {
    CData,
    Text,
}

fn write_element(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    content: &str,
    content_type: ContentType,
) -> Result<(), Error> {
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    match content_type {
        ContentType::CData => writer.write_event(Event::CData(BytesCData::new(content)))?,
        ContentType::Text => writer.write_event(Event::Text(BytesText::new(content)))?,
    }
    writer.write_event(Event::End(BytesEnd::new(tag)))?;

    Ok(())
}

fn write_kml(content: &Vec<u8>, filename: &str) -> Result<(), Error> {
    create_parent_directory(filename)?;
    let mut f = File::create(filename)?;
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
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    let mut root = BytesStart::new("kml");
    root.push_attribute(("xmlns", "http://www.opengis.net/kml/2.2"));
    writer.write_event(Event::Start(root))?;
    writer.write_event(Event::Start(BytesStart::new("Document")))?;
    write_element(
        &mut writer,
        "name",
        Path::new(filename).file_stem().unwrap().to_str().unwrap(),
        ContentType::Text,
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
            "<p><b>Filename:</b> {}</p>
            <p><b>Longitude:</b> {}</p>
            <p><b>Latitude:</b> {}</p>
            <p><b>Altitude:</b> {}</p>",
            file_name!(image),
            longitude,
            latitude,
            altitude
        );
        writer.write_event(Event::Start(BytesStart::new("Placemark")))?;
        write_element(
            &mut writer,
            "name",
            &(idx + 1).to_string(),
            ContentType::Text,
        )?;
        write_element(&mut writer, "description", &description, ContentType::CData)?;
        writer.write_event(Event::Start(BytesStart::new("Point")))?;
        write_element(
            &mut writer,
            "coordinates",
            &format!("{},{},{}", longitude, latitude, altitude),
            ContentType::Text,
        )?;
        writer.write_event(Event::End(BytesEnd::new("Point")))?;
        writer.write_event(Event::End(BytesEnd::new("Placemark")))?;
        count[2] += 1;
    }

    writer.write_event(Event::End(BytesEnd::new("Document")))?;
    writer.write_event(Event::End(BytesEnd::new("kml")))?;

    let result = writer.into_inner();
    write_kml(&result, filename)?;
    debug!("Successfully generated kml for {} images,skipped {} images due to invalid_exif and {} images due to no GPS information",count[2],count[0],count[1]);
    Ok(())
}
