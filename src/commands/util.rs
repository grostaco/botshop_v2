use std::ops::Index;

use chrono::{DateTime, Duration, Utc};

type Record = (String, u8, Option<i64>);
pub struct Records(Vec<Record>);
pub struct RecordsIter<'a> {
    records: &'a Records,
    index: usize,
}

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

    pub fn iter(&self) -> RecordsIter<'_> {
        RecordsIter {
            records: self,
            index: 0,
        }
    }
}

impl Index<usize> for Records {
    type Output = Record;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a> Iterator for RecordsIter<'a> {
    type Item = &'a Record;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.records.len() {
            self.index += 1;
            Some(&self.records[self.index - 1])
        } else {
            None
        }
    }
}

impl IntoIterator for Records {
    type Item = Record;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub fn get_tomorrow() -> DateTime<Utc> {
    (Utc::now() + Duration::days(1)).date().and_hms(0, 0, 0)
}

pub fn get_today() -> DateTime<Utc> {
    Utc::now().date().and_hms(0, 0, 0)
}
