use anyhow::Result;
use sqlite::Connection;

use crate::infra::{WrapOption, WrapResult};

pub struct Cache {
    conn: Connection,
}

impl Cache {
    pub fn new(path: &str) -> Result<Self> {
        let conn = sqlite::open(path)?;
        let init = "\
            CREATE TABLE IF NOT EXISTS hashes (\
                path TEXT NOT NULL,\
                hash BLOB NOT NULL\
            );\
        ";
        conn.execute(init)?;
        Self { conn }.wrap_ok()
    }

    pub fn query(&self, img_path: &str) -> Result<Option<Box<[u8]>>> {
        let stmt = "SELECT hash FROM hashes WHERE path = ?";
        let mut stmt = self.conn.prepare(stmt).expect("Failed to prepare stmt");
        stmt.bind((1, img_path)).expect("Failed to bind param");
        let row = stmt.into_iter().next();
        match row {
            Some(Ok(row)) => {
                let hash: &[u8] = row.read("hash");
                hash.to_vec().into_boxed_slice().wrap_some().wrap_ok()
            }
            Some(Err(e)) => Err(e)?,
            _ => None.wrap_ok(),
        }
    }

    pub fn insert(&self, img_path: &str, hash: &[u8]) -> Result<()> {
        let stmt = "INSERT INTO hashes (path, hash) VALUES (?, ?)";
        let mut stmt = self.conn.prepare(stmt).expect("Failed to prepare stmt");
        stmt.bind((1, img_path)).expect("Failed to bind param");
        stmt.bind((2, hash)).expect("Failed to bind param");
        stmt.next()?;
        Ok(())
    }
}
