use std::{fs, fs::OpenOptions, io::BufWriter};
use std::str::FromStr;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{PasswordHashString, SaltString};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use openssl::base64;
use rocket::serde;
use rocket::serde::json::serde_json;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::ser::SerializeStruct;
use crate::ApiError;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Settings {
    #[serde(default)]
    mail: Mail,
    #[serde(default)]
    common: Common,
    #[serde(default)]
    auth: Auth
}
pub struct FrontendSettings(pub Settings);

impl Serialize for FrontendSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Settings", 1)?;
        state.serialize_field("common", &self.0.common)?;
        state.serialize_field("mail", &self.0.mail)?;
        state.end()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Common {
    username: String
}

impl Default for Common {
    fn default() -> Self {
        Self{ username: String::from_str("admin").unwrap()}
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Mail {
    address: String,
    username: Option<String>,
    password: Option<String>,
    from: String,
    to: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Auth {
    jwt_key: String,
    password_hash: Option<String>
}

fn generate_jwt_key() -> String {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    base64::encode_block(&secret)
}

impl Default for Auth {
    fn default() -> Self {
        Self{ jwt_key: generate_jwt_key(), password_hash: Default::default() }
    }
}

const FILE_PATH: &str = "settings.json";

impl Settings {
    pub fn new(file_path: Option<&str>) -> Result<Self, ApiError> {
        let settings_string = fs::read_to_string(file_path.unwrap_or(FILE_PATH))
            .unwrap_or("{}".to_string());
        let settings: Self = serde_json::from_str(&settings_string).unwrap_or(Default::default());
        settings.save(None)?;
        Ok(settings)
    }

    pub fn save(&self, file_path: Option<&str>) -> Result<(), ApiError> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path.unwrap_or(FILE_PATH))
            .map_err(|_| ApiError::Other("Failed to save settings".to_string()))?;
        let writer = BufWriter::new(f);
        serde_json::to_writer_pretty(writer, self).map_err(|_| ApiError::Other("Failed to save settings".to_string()))
    }

    pub fn set_settings(&mut self, settings: &Settings) -> Result<(), ApiError> {
        self.common = settings.common.clone();
        self.mail = settings.mail.clone();
        self.save(None).map_err(|_| { ApiError::Other("Failed to save username".to_string()) } )
    }

    pub fn get_jwt_key(&self) -> Vec<u8> {
        base64::decode_block(self.auth.jwt_key.as_str()).expect("JWT key is malformed")
    }

    pub fn set_password(&mut self, password: &str) -> Result<(), ApiError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|_| ApiError::Other("Failed to hash password".to_string()))?;
        self.auth.password_hash = Some(password_hash.to_string());
        self.save(None)?;
        Ok(())
    }

    pub fn get_password_hash(&self) -> Result<PasswordHashString, ApiError> {
        if self.auth.password_hash.is_none() { return Err(ApiError::Other("Password not configured".to_string())) }
        let password_string = self.auth.password_hash.clone().unwrap();
        PasswordHashString::new(&password_string).map_err(|_| ApiError::Other("Password has invalid format".to_string()))
    }

    pub fn is_setup(&self) -> bool {
        self.auth.password_hash.is_some()
    }
    
    pub fn set_username(&mut self, username: &String) -> Result<(), ApiError> {
        self.common.username = username.clone();
        self.save(None).map_err(|_| { ApiError::Other("Failed to save username".to_string()) } )
    }
}