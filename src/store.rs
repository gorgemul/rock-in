use chrono::{DateTime, Local};
use rusqlite::{Connection, Result, params};

const DB_NAME: &str = "rockin.db";

#[derive(Debug)]
pub struct Store {
    conn: Connection,
}

pub struct Session {
    _id: i64,
    duration_seconds: i64,
    created_at: DateTime<Local>,
}

impl Session {
    pub fn duration_seconds(&self) -> u64 {
        self.duration_seconds as u64
    }
    pub fn created_at(&self) -> String {
        self.created_at.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

impl Store {
    pub fn new() -> Result<Self> {
        let conn = Connection::open(DB_NAME)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions(
            id INTEGER PRIMARY KEY,
            duration_seconds INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)",
            (),
        )?;
        Ok(Self { conn })
    }
    pub fn save(&mut self, duration: u64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (duration_seconds) VALUES (?1)",
            params![&(duration as i32)],
        )?;
        Ok(())
    }
    pub fn find_all(&self) -> Result<Vec<Session>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, duration_seconds, created_at FROM sessions")?;
        let sessions = stmt
            .query_map([], |row| {
                Ok(Session {
                    _id: row.get(0)?,
                    duration_seconds: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(sessions)
    }
}
