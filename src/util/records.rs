use core::slice;
use std::ops::Index;

use serde::{Deserialize, Serialize};

pub type Record = (String, u8, Option<i64>);
#[derive(Debug)]
pub struct Records(Vec<Record>);

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct RecordWrite {
    pub task: String,
    pub points: u8,
    pub completed: Option<i64>,
}

#[derive(Serialize)]
pub struct RecordRow<'a> {
    pub task: &'a str,
    pub points: u8,
    pub completed: Option<i64>,
}

impl Records {
    pub fn new() -> Self {
        Records(Vec::new())
    }
    pub fn from_file(path: &str) -> Result<Self, csv::Error> {
        let rdr = csv::Reader::from_path(path)?;
        let mut records = Records::new();

        for record in rdr.into_deserialize() {
            let record: RecordWrite = record?;
            records
                .0
                .push((record.task, record.points, record.completed));
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
