use clap::ValueEnum;
use log::trace;
use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    ///Off
    Off,
    ///Error
    Error,
    /// Warning
    Warning,
    /// Info
    Info,
    /// Debug
    Debug,
    /// trace
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, PartialEq)]
pub enum GPSInformationField {
    Char(char),
    Float(f64),
    Int(u8),
}

impl fmt::Display for GPSInformationField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GPSInformationField::Char(data) => data.fmt(f),
            GPSInformationField::Float(data) => data.fmt(f),
            GPSInformationField::Int(data) => data.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct GPSInformation {
    pub altitude: GPSInformationField,
    ///0 = Above Sea Level
    ///1 = Below Sea Level
    pub altitude_ref: GPSInformationField,
    pub latitude: GPSInformationField,
    ///'N' = North
    ///'S' = South
    pub latitude_ref: GPSInformationField,
    pub longitude: GPSInformationField,
    ///'E' = East
    ///'W' = West
    pub longitude_ref: GPSInformationField,
    modified: [bool; 6],
}

impl GPSInformation {
    #[allow(dead_code)]
    pub fn get_index(&self, i: &usize) -> &GPSInformationField {
        match i {
            0 => &self.altitude,
            1 => &self.altitude_ref,
            2 => &self.latitude,
            3 => &self.latitude_ref,
            4 => &self.longitude,
            5 => &self.longitude_ref,
            _ => panic!("unknown field requested: {}", i),
        }
    }
    pub fn get_param(&self, param: &str) -> String {
        match param {
            "alt" | "altitude" => {
                format!("{:.2}", self.altitude)
            }
            "lat" | "latitude" => {
                if self.latitude_ref == GPSInformationField::Char('E') {
                    format!("-{:.6}", self.latitude)
                } else {
                    format!("{:.6}", self.latitude)
                }
            }
            "lon" | "longitude" => {
                if self.longitude_ref == GPSInformationField::Char('S') {
                    format!("-{:.6}", self.longitude)
                } else {
                    format!("{:.6}", self.longitude)
                }
            }
            _ => panic!("unknown field requested: {}", param),
        }
    }
    pub fn set_index(&mut self, i: &usize, value: GPSInformationField) {
        match i {
            0 => self.altitude = value,
            1 => self.altitude_ref = value,
            2 => self.latitude = value,
            3 => self.latitude_ref = value,
            4 => self.longitude = value,
            5 => self.longitude_ref = value,
            _ => panic!("unknown field: {}", i),
        }
        self.modified[*i] = true;
    }
    pub fn is_valid(&self) -> bool {
        let mut valid = true;
        for m in self.modified[2..6].iter() {
            valid &= m;
        }
        trace!(
            "Checking validity, modified array: {:?}, Valid: {}",
            self.modified,
            valid
        );
        valid
    }
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for GPSInformation {
    fn default() -> Self {
        GPSInformation {
            altitude: GPSInformationField::Float(0.),
            altitude_ref: GPSInformationField::Int(0),
            latitude: GPSInformationField::Float(0.),
            latitude_ref: GPSInformationField::Char(0 as char),
            longitude: GPSInformationField::Float(0.),
            longitude_ref: GPSInformationField::Char(0 as char),
            modified: [false, false, false, false, false, false],
        }
    }
}

impl fmt::Display for GPSInformation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ExifReadError(exif::Error),
    QuickXMLError(quick_xml::Error),
    LoggerError(log::SetLoggerError),
}

impl Display for Error {
    // 3
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<exif::Error> for Error {
    fn from(error: exif::Error) -> Self {
        Error::ExifReadError(error)
    }
}

impl From<quick_xml::Error> for Error {
    fn from(error: quick_xml::Error) -> Self {
        Error::QuickXMLError(error)
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(error: log::SetLoggerError) -> Self {
        Error::LoggerError(error)
    }
}
