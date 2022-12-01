use std::{
    fs::File,
    io::{BufReader, Write},
    path::Path,
};

use exif::{In, Reader, Tag, Value};
use log::{debug, error};

// use kml::{
//     types::{AltitudeMode, Coord, Point},
//     Kml, KmlWriter,
// };
// use quick_xml;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Writer,
};

use super::structures::{GPSInformation, GPSInformationField};

pub fn get_gps_information(image: &str) -> GPSInformation {
    let file = File::open(image).unwrap();
    let exif = Reader::new()
        .read_from_container(&mut BufReader::new(&file))
        .unwrap();
    let tag_list = [
        Tag::GPSAltitude,
        Tag::GPSAltitudeRef,
        Tag::GPSLatitude,
        Tag::GPSLatitudeRef,
        Tag::GPSLongitude,
        Tag::GPSLongitudeRef,
    ];
    let mut gps_info = GPSInformation::new();
    for (idx, &tag) in tag_list.iter().enumerate() {
        if let Some(field) = exif.get_field(tag, In::PRIMARY) {
            debug!("Field value for tag {}:{:?}", tag, field.value);
            match field.value {
                Value::Rational(ref vec) if !vec.is_empty() => {
                    if vec.len() == 1 {
                        gps_info.set(&idx, GPSInformationField::Float(vec[0].to_f64()))
                    } else {
                        gps_info.set(
                            &idx,
                            GPSInformationField::Float(
                                vec[0].to_f64()
                                    + vec[1].to_f64() / 60 as f64
                                    + vec[2].to_f64() / 3600 as f64,
                            ),
                        )
                    }
                }
                Value::Byte(ref byte) if !byte.is_empty() => {
                    gps_info.set(&idx, GPSInformationField::Int(byte[0]))
                }
                Value::Ascii(ref vec) if !vec.is_empty() => {
                    gps_info.set(&idx, GPSInformationField::Char(vec[0][0] as char))
                }
                _ => error!(
                    "Invalid value encountered for tag {}:{:?}",
                    tag, field.value
                ),
            }
        }
    }
    gps_info
}

fn write_element(
    writer: &mut Writer<Vec<u8>>,
    tag: &str,
    content: &str,
) -> Result<(), quick_xml::Error> {
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    writer.write_event(Event::Text(BytesText::new(content)))?;
    writer.write_event(Event::End(BytesEnd::new(tag)))?;

    Ok(())
}

fn write_kml(content: &Vec<u8>, filename: &str) -> Result<(), std::io::Error> {
    let mut f = File::create(filename)?;
    f.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    f.write_all(content)?;
    Ok(())
}

pub fn generate_kml_from_images(
    images: &Vec<String>,
    filename: &str,
) -> Result<(), quick_xml::Error> {
    let buf = Vec::new();
    let mut writer = Writer::new_with_indent(buf, ' ' as u8, 4);

    let mut root = BytesStart::new("kml");
    root.push_attribute(("xmlns", "http://www.opengis.net/kml/2.2"));
    writer.write_event(Event::Start(root))?;
    writer.write_event(Event::Start(BytesStart::new("Folder")))?;
    write_element(
        &mut writer,
        "name",
        Path::new(filename).file_stem().unwrap().to_str().unwrap(),
    )?;

    for image in images {
        let gps_information = get_gps_information(image);
        if gps_information.is_valid() {
            let altitude = gps_information.get_param("alt");
            let longitude = gps_information.get_param("lon");
            let latitude = gps_information.get_param("lat");
            let name = Path::new(image).file_name().unwrap().to_str().unwrap();
            let description = "";
            writer.write_event(Event::Start(BytesStart::new("Placemark")))?;
            write_element(&mut writer, "name", name)?;
            write_element(&mut writer, "description", description)?;
            writer.write_event(Event::Start(BytesStart::new("Point")))?;
            write_element(
                &mut writer,
                "coordinates",
                &format!("{},{},{}", longitude, latitude, altitude),
            )?;
            writer.write_event(Event::End(BytesEnd::new("Point")))?;
            writer.write_event(Event::End(BytesEnd::new("Placemark")))?;
        }
    }

    writer.write_event(Event::End(BytesEnd::new("Folder")))?;
    writer.write_event(Event::End(BytesEnd::new("kml")))?;

    let result = writer.into_inner();
    write_kml(&result, filename)?;
    Ok(())
}
