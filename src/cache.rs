use std::fs;

use anyhow::Result;
use colored::Colorize;
use sqlite::Connection;

use crate::fmt_path_for_display::fmt_path_for_display;

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
        Ok(Self { conn })
    }

    pub fn query(&self, img_path: &str) -> Result<Option<Box<[u8]>>> {
        let stmt = "SELECT hash FROM hashes WHERE path = ?";
        let mut stmt = self.conn.prepare(stmt).expect("Failed to prepare stmt");
        stmt.bind((1, img_path)).expect("Failed to bind param");
        let row = stmt.into_iter().next();
        match row {
            Some(Ok(row)) => {
                let hash: &[u8] = row.read("hash");
                Ok(Some(hash.to_vec().into_boxed_slice()))
            }
            Some(Err(e)) => Err(e)?,
            _ => Ok(None),
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

    fn delete(&self, img_path: &str) -> Result<()> {
        let stmt = "DELETE FROM hashes WHERE path = ?";
        let mut stmt = self.conn.prepare(stmt).expect("Failed to prepare stmt");
        stmt.bind((1, img_path)).expect("Failed to bind param");
        stmt.next()?;
        Ok(())
    }

    pub fn gc(&self) {
        let stmt = "SELECT path FROM hashes";
        let stmt = self.conn.prepare(stmt).expect("Failed to prepare stmt");
        stmt.into_iter().for_each(|row| match row {
            Ok(row) => {
                let img_path = row.read("path");
                if fs::metadata(img_path).is_err() {
                    if let Err(e) = self.delete(img_path) {
                        println!(
                            "{} Failed to delete cache in database: {} [{}]",
                            "[ERR]".red(),
                            img_path,
                            e
                        );
                    } else {
                        let display_path = fmt_path_for_display(img_path, 10);
                        println!("{} {}", "[GC]".blue(), display_path);
                    }
                }
            }
            Err(e) => println!("{} Failed to query cache in database: {}", "[ERR]".red(), e),
        });
    }
}
