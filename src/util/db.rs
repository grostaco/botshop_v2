use rusqlite::{
    params,
    types::{FromSql, ToSqlOutput},
    Connection, Result, ToSql,
};
use serde::{Deserialize, Serialize};
pub type Record = (String, u8, Option<i64>);

#[derive(Debug, Deserialize, Serialize)]
pub struct Records(pub Vec<Record>);

impl ToSql for Records {
    fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>> {
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

impl Records {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push_record(&mut self, task: String, points: u8, timestamp: Option<i64>) {
        self.0.push((task, points, timestamp));
    }
}

#[derive(Debug)]
pub struct User {
    pub id: u64,
    pub daily: Records,
    pub periodic: Records,
    pub transactions: Records,
}

impl User {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            daily: Records::new(),
            periodic: Records::new(),
            transactions: Records::new(),
        }
    }
}

pub fn insert_user(db_path: &str, user: User) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.prepare(
        "CREATE TABLE IF NOT EXISTS users (
                        id              INTEGER PRIMARY KEY,
                        daily           BLOB,
                        periodic        BLOB,
                        transactions    BLOB)",
    )?
    .execute([])?;

    let mut stmt =
        conn.prepare("INSERT INTO users (id,daily,periodic,transactions) VALUES (?1, ?2, ?3, ?4)")?;
    stmt.execute(params![
        user.id,
        user.daily,
        user.periodic,
        user.transactions
    ])?;

    Ok(())
}

pub fn update_user(db_path: &str, user: User) -> Result<()> {
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(&format!(
        "UPDATE users SET daily=?1,
                          periodic=?2,
                          transactions=?3
            WHERE id={}",
        user.id
    ))?;

    stmt.execute(params![user.daily, user.periodic, user.transactions])?;

    Ok(())
}

pub fn query_user(db_path: &str, id: u64) -> Result<Option<User>> {
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id,daily,periodic,transactions FROM users WHERE id=?")?;
    let user = stmt
        .query_map([id], |row| {
            Ok(User {
                id: row.get(0)?,
                daily: row.get(1)?,
                periodic: row.get(2)?,
                transactions: row.get(3)?,
            })
        })?
        .into_iter()
        .map(|row| row.unwrap())
        .last();

    Ok(user)
}

#[cfg(test)]
mod test {
    use super::{insert_user, query_user, update_user, Records, User};

    #[test]
    fn try_insert() {
        insert_user(
            "x.db",
            User {
                id: 0,
                daily: Records::new(),
                periodic: Records::new(),
                transactions: Records::new(),
            },
        )
        .expect("??");
    }

    #[test]
    fn try_query() {
        query_user("x.db", 0).unwrap();
    }

    #[test]
    fn try_update() {
        let mut periodic = Records::new();
        periodic.push_record("A".to_owned(), 5, Some(5));
        update_user(
            "x.db",
            User {
                id: 0,
                daily: Records::new(),
                periodic: periodic,
                transactions: Records::new(),
            },
        )
        .unwrap();
    }
}
