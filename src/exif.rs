use chrono::prelude::*;
use exif::{Reader, Tag};

use std::fs::File;
use std::io::BufReader;
use std::path;

pub struct Exif {
    reader: Reader,
    utc: FixedOffset,
    zero: DateTime<FixedOffset>,
}

impl Exif {
    pub fn from_path(path: &path::Path) -> Result<Exif, String> {
        let utc = FixedOffset::east(0);
        let file = File::open(path).unwrap();
        let reader = Reader::new(&mut BufReader::new(&file));
        match reader {
            Err(err) => Err(format!("{:?}", err)),
            Ok(rd) => Ok(Exif {
                reader: rd,
                utc: utc,
                zero: utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
            }),
        }
    }
    pub fn get_date(&self) -> DateTime<FixedOffset> {
        let s = self
            .reader
            .get_field(Tag::DateTimeOriginal, false)
            .unwrap()
            .value
            .display_as(Tag::DateTimeOriginal)
            .to_string();

        let file_time = self
            .utc
            .datetime_from_str(&s, "%Y-%m-%d %H:%M:%S")
            .unwrap_or(self.zero);
        return file_time;
    }
}
