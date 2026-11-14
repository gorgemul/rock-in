use std::path::PathBuf;

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
    fn db_path() -> PathBuf {
        let dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rock-in");
        if let Err(err) = std::fs::create_dir_all(&dir) {
            eprintln!("Create dir fail: {}", err);
        }
        dir.join(DB_NAME)
    }

    pub fn new() -> Result<Self> {
        let conn = Connection::open(Self::db_path())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions(
            id INTEGER PRIMARY KEY,
            duration_seconds INTEGER NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)",
            (),
        )?;
        // NOTE: We are not handling situation when we are ticking the clock, but the program is being
        // terminated in midway. So we just simply clean those problemetic entries right now, maybe
        // catch those terminated signal and call end_session to save that session.
        if let Ok(effected_rows) =
            conn.execute("DELETE FROM sessions WHERE duration_seconds IS NULL", ())
            && effected_rows > 0
        {
            eprintln!("{effected_rows} broken sessions being deleted");
        }
        Ok(Self { conn })
    }
    pub fn start_session(&self) -> Result<()> {
        self.conn
            .execute("INSERT INTO sessions DEFAULT VALUES", ())?;
        Ok(())
    }
    pub fn end_session(&self, duration_seconds: u64) -> Result<()> {
        if let Ok(effected_rows) = self.conn.execute(
            "UPDATE sessions SET duration_seconds = ?1 WHERE duration_seconds IS NULL",
            params![&(duration_seconds as i32)],
        ) && effected_rows != 1
        {
            eprintln!("Should only save one session but {effected_rows}");
        }
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
