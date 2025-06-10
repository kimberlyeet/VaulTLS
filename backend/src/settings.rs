use std::{env, fs, fs::OpenOptions, io::BufWriter};
use std::env::VarError;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use openssl::base64;
use rocket::serde;
use rocket::serde::json::serde_json;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::ser::SerializeStruct;
use crate::ApiError;
use crate::data::enums::MailEncryption;
use crate::constants::SETTINGS_FILE_PATH;

/// Settings for the backend.
#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct Settings {
    #[serde(default)]
    mail: Mail,
    #[serde(default)]
    common: Common,
    #[serde(default)]
    auth: Auth,
    #[serde(default)]
    oidc: OIDC
}

/// Wrapper for the settings to make them serializable for the frontend.
pub(crate) struct FrontendSettings(pub(crate) Settings);

/// Serialize the settings for the frontend.
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

/// Common settings for the backend.
#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct Common {
    password_enabled: bool
}

/// Mail settings for the backend.
#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct Mail {
    pub(crate) smtp_host: String,
    pub(crate) smtp_port: u16,
    pub(crate) encryption: MailEncryption,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) from: String
}

impl Mail {
    /// Check if the mail settings are valid.
    pub(crate) fn is_valid(&self) -> bool {
        self.smtp_host.len() > 0 && self.smtp_port > 0 && self.from.len() > 0
    }
}

/// Authentication settings for the backend.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Auth {
    jwt_key: String,
}

/// OpenID Connect settings for the backend.
#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct OIDC {
    pub(crate) id: String,
    pub(crate) secret: String,
    pub(crate) auth_url: String,
    pub(crate) callback_url: String
}

impl Default for Auth {
    fn default() -> Self {
        Self{ jwt_key: generate_jwt_key(), }
    }
}

/// Generates a new JWT key.
fn generate_jwt_key() -> String {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    base64::encode_block(&secret)
}

/// Fills the OIDC config with the environment variables.
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

impl Settings {
    /// Load saved settings from a file
    pub(crate) fn load_from_file(file_path: Option<&str>) -> Result<Self, ApiError> {
        let settings_string = fs::read_to_string(file_path.unwrap_or(SETTINGS_FILE_PATH))
            .unwrap_or("{}".to_string());
        let mut settings: Self = serde_json::from_str(&settings_string).unwrap_or(Default::default());
        settings.oidc = fill_oidc_config();
        settings.save_to_file(None)?;
        Ok(settings)
    }

    /// Save current settings to a file
    pub(crate) fn save_to_file(&self, file_path: Option<&str>) -> Result<(), ApiError> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path.unwrap_or(SETTINGS_FILE_PATH))
            .map_err(|_| ApiError::Other("Failed to save settings".to_string()))?;
        let writer = BufWriter::new(f);
        serde_json::to_writer_pretty(writer, self).map_err(|_| ApiError::Other("Failed to save settings".to_string()))
    }
    
    /// Set the settings and save them to the file.
    pub(crate) fn set_settings(&mut self, settings: &Settings) -> Result<(), ApiError> {
        self.common = settings.common.clone();
        self.mail = settings.mail.clone();
        self.oidc = settings.oidc.clone();

        self.save_to_file(None)
    }
    
    /// Get the JWT key from the settings.
    pub(crate) fn get_jwt_key(&self) -> Result<Vec<u8>, ApiError> {
        base64::decode_block(self.auth.jwt_key.as_str())
            .map_err(|_| ApiError::Other("Failed to decode jwt key".to_string()))
    }
    
    pub(crate) fn get_mail(&self) -> &Mail { &self.mail }
    pub(crate) fn get_oidc(&self) -> &OIDC { &self.oidc }
    
    /// Check if the password is enabled.
    pub(crate) fn password_enabled(&self) -> bool {
        self.common.password_enabled
    }
}