use super::Records;
use rusqlite::{params, Connection, Result};
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

    pub fn from_file(db_file: &str, user_id: u64) -> Result<Self> {
        match query_user(db_file, user_id)? {
            None => {
                insert_user(db_file, User::new(user_id))?;
                Ok(Self::new(user_id))
            }
            Some(user) => Ok(user),
        }
    }

    pub fn update(&self, db_path: &str) -> Result<()> {
        update_user(db_path, self)?;
        Ok(())
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

pub fn update_user(db_path: &str, user: &User) -> Result<()> {
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
