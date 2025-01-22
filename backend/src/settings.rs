use std::{fs::OpenOptions, io::{BufReader, BufWriter}};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    #[serde(default)]
    mail: Mail,
    #[serde(default)]
    common: Common
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Common {
    #[serde(default = "default_username")]
    username: String
}

fn default_username() -> String {
    "admin".to_string()
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Mail {
    address: String,
    username: Option<String>,
    password: Option<String>,
    from: String,
    to: String
}

const FILE_PATH: &str = "settings.json";

impl Settings {
    pub fn new(file_path: Option<&str>) -> Result<Self, anyhow::Error> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path.unwrap_or(FILE_PATH))?;
        let reader = BufReader::new(f);
        let settings: Self = serde_json::from_reader(reader).unwrap_or(Self::default());
        Ok(settings)
    }

    pub fn save(&self, file_path: Option<&str>) -> Result<(), anyhow::Error> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path.unwrap_or(FILE_PATH))?;
        let writer = BufWriter::new(f);
        Ok(serde_json::to_writer_pretty(writer, self)?)
    }

    pub fn set_settings(&mut self, settings: &Settings) -> Result<(), anyhow::Error> {
        self.common = settings.common.clone();
        self.mail = settings.mail.clone();
        self.save(None)
    }
}