use std::path::Path;

use rusqlite::{params, Connection, Result};

use crate::Certificate;

pub struct CertificateDB {
    connection: Connection
}

impl CertificateDB {
    // Initialize the database
    pub fn new(db_path: &Path) -> Result<Self> {
        let connection = Connection::open(db_path)?;

        Ok(Self { connection })
    }

    pub fn initialize_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.connection.execute(
            "CREATE TABLE ca_certificates (
                id INTEGER PRIMARY KEY,
                created_on INTEGER NOT NULL,
                valid_until INTEGER NOT NULL,
                certificate BLOB,
                key BLOB
            )",
            [],
        )?;

        self.connection.execute(
            "CREATE TABLE user_certificates (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                created_on INTEGER NOT NULL,
                valid_until INTEGER NOT NULL,
                pkcs12 BLOB
            )",
            [],
        )?;

        Ok(())
    }    

    pub fn insert_ca(
        &self, ca: Certificate
    ) -> Result<(), rusqlite::Error> {
        self.connection.execute(
            "INSERT INTO ca_certificates (created_on, valid_until, certificate, key) VALUES (?1, ?2, ?3, ?4)",
            params![ca.created_on, ca.valid_until, ca.cert, ca.key],
        )?;

        Ok(())
    }

    pub fn get_current_ca(&self) -> Result<Certificate, rusqlite::Error> {
    
        // Query to fetch the last row
        let mut stmt = self.connection.prepare("SELECT * FROM ca_certificates ORDER BY id DESC LIMIT 1")?;
    
        // Execute the query and retrieve the row
        stmt.query_row([], |row| {
            Ok(Certificate{
                id: row.get(0)?,
                created_on: row.get(1)?,
                valid_until: row.get(2)?,
                cert: row.get(3)?,
                key: row.get(4)?,
                ..Default::default()
            })
        })
    }

    pub fn get_all_user_cert(&self) -> Result<Vec<Certificate>, rusqlite::Error>{
        let mut stmt = self.connection
            .prepare("SELECT id, name, created_on, valid_until, pkcs12 FROM user_certificates")?;

        let x = Ok(stmt
            .query_map([], |row| {
                Ok(Certificate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_on: row.get(2)?,
                    valid_until: row.get(3)?,
                    pkcs12: row.get(4)?,
                    ..Default::default()
                })
            })?
            .map(|res| res.unwrap())
            .collect()
        ); x
    }

    pub fn get_user_pkcs12(&self, id: i64) -> Result<Vec<u8>, rusqlite::Error> {
        let mut stmt = self.connection.prepare("SELECT pkcs12 FROM user_certificates WHERE id = ?1")?;

        stmt.query_row(
            params![id],
            |row| row.get(0),
        )
    }

    pub fn insert_user_cert(&self, cert: Certificate) -> Result<i64, rusqlite::Error> {
        self.connection.execute(
            "INSERT INTO user_certificates (name, created_on, valid_until, pkcs12) VALUES (?1, ?2, ?3, ?4)",
            params![cert.name, cert.created_on, cert.valid_until, cert.pkcs12],
        )?;

        Ok(self.connection.last_insert_rowid())
    }

    pub fn delete_user_cert(&self, id: i64) -> Result<(), rusqlite::Error> {
        self.connection.execute(
            "DELETE FROM user_certificates WHERE id=?1",
            params![id]
        )?;

        Ok(())
    }
}