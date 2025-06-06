use std::{env, fs, fs::OpenOptions, io::BufWriter};
use std::env::VarError;
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
    auth: Auth,
    #[serde(default)]
    oidc: OIDC
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
        state.serialize_field("oidc", &self.0.oidc)?;
        state.end()
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Common {
    password_enabled: bool
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
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct OIDC {
    pub id: String,
    pub secret: String,
    pub auth_url: String,
    pub callback_url: String
}

impl Default for Auth {
    fn default() -> Self {
        Self{ jwt_key: generate_jwt_key(), }
    }
}

fn generate_jwt_key() -> String {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    base64::encode_block(&secret)
}

fn fill_oidc_config() -> OIDC {
    let get_env = || -> Result<OIDC, VarError> {
        let id = env::var("OIDC_ID")?;
        let secret = env::var("OIDC_SECRET")?;
        let auth_url = env::var("OIDC_AUTH_URL")?;
        let callback_url = env::var("OIDC_CALLBACK_URL")?;
        Ok(OIDC{ id, secret, auth_url, callback_url })
    };

    get_env().unwrap_or_else(|_| OIDC::default())
}



const FILE_PATH: &str = "settings.json";

impl Settings {
    pub fn new(file_path: Option<&str>) -> Result<Self, ApiError> {
        let settings_string = fs::read_to_string(file_path.unwrap_or(FILE_PATH))
            .unwrap_or("{}".to_string());
        let mut settings: Self = serde_json::from_str(&settings_string).unwrap_or(Default::default());
        settings.oidc = fill_oidc_config();
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
        self.oidc = settings.oidc.clone();

        self.save(None)
    }

    pub fn get_jwt_key(&self) -> Vec<u8> {
        base64::decode_block(self.auth.jwt_key.as_str()).expect("JWT key is malformed")
    }

    pub fn get_oidc(&self) -> &OIDC {
        &self.oidc
    }
    
    pub fn password_enabled(&self) -> bool {
        self.common.password_enabled
    }
}