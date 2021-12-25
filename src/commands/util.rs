use core::slice;
use std::ops::Index;

use chrono::{DateTime, Duration, Utc};

pub type Record = (String, u8, Option<i64>);
pub struct Records(Vec<Record>);

impl Records {
    pub fn new() -> Self {
        Records(Vec::new())
    }
    pub fn from_file(path: &str) -> Result<Self, csv::Error> {
        let mut rdr = csv::Reader::from_path(path)?;
        let mut records = Records::new();

        for record in rdr.records() {
            let record = record?;
            records.0.push((
                record.get(0).expect("Expected task name").to_owned(),
                record
                    .get(1)
                    .expect("Expected points")
                    .parse()
                    .expect("Expected point as an integer"),
                match record.get(2).expect("Expected time completed") {
                    "None" => None,
                    timestamp => Some(
                        timestamp
                            .parse()
                            .expect("Expected timestamp as either an integer or None"),
                    ),
                },
            ));
        }

        Ok(records)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> slice::Iter<'_, Record> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<'_, Record> {
        self.0.iter_mut()
    }
}

impl Index<usize> for Records {
    type Output = Record;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IntoIterator for Records {
    type Item = Record;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Records {
    type Item = &'a Record;

    type IntoIter = slice::Iter<'a, Record>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Records {
    type Item = &'a mut Record;

    type IntoIter = slice::IterMut<'a, Record>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

pub fn get_tomorrow() -> DateTime<Utc> {
    (Utc::now() + Duration::days(1)).date().and_hms(0, 0, 0)
}

pub fn get_today() -> DateTime<Utc> {
    Utc::now().date().and_hms(0, 0, 0)
}
