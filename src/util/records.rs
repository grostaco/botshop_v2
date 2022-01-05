use core::slice;
use std::ops::Index;

use rusqlite::{
    types::{FromSql, ToSqlOutput},
    ToSql,
};
use serde::{Deserialize, Serialize};

pub type Record = (String, i64, Option<i64>);
#[derive(Debug, Serialize, Deserialize)]
pub struct Records(pub Vec<Record>);

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct RecordWrite {
    pub task: String,
    pub points: i64,
    pub completed: Option<i64>,
}

#[derive(Serialize)]
pub struct RecordRow<'a> {
    pub task: &'a str,
    pub points: i64,
    pub completed: Option<i64>,
}

impl Records {
    pub fn new() -> Self {
        Records(Vec::new())
    }

    pub fn push(&mut self, task: String, points: i64, timestamp: Option<i64>) {
        self.0.push((task, points, timestamp))
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

impl ToSql for Records {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(
            bincode::serialize(self).expect("Unable to serialize Records"),
        ))
    }
}

impl FromSql for Records {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(bincode::deserialize(value.as_blob().unwrap()).unwrap())
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
