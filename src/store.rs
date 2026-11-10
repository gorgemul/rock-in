use rusqlite::{Connection, Result, params};

const DB_NAME: &str = "rockin.db";

#[derive(Debug)]
pub struct Store {
    conn: Connection,
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
}
