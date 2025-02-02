use crate::settings::Settings;
use crate::{ApiError, AppState};
use argon2::{Argon2, PasswordVerifier};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Authenticated;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authenticated {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = match request.headers().get_one("Authorization") {
            Some(h) if h.starts_with("Bearer ") => &h["Bearer ".len()..],
            _ => return Outcome::Error((Status::Unauthorized, ())),
        };

        let config = match request.rocket().state::<AppState>() {
            Some(c) => c,
            None => return Outcome::Error((Status::InternalServerError, ())),
        };

        let settings = config.settings.lock().await;
        let jwt_key = settings.get_jwt_key();
        let decoding_key = DecodingKey::from_secret(&jwt_key);
        let validation = Validation::default();

        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(_) => Outcome::Success(Authenticated),
            Err(_) => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}

pub fn verify_password(settings: &Settings, password: &str) -> Result<String, ApiError> {
    let jwt_key = settings.get_jwt_key();
    let password_hash_string = settings.get_password_hash()
        .map_err(|_| ApiError::Unauthorized(Some("Password has not been set".to_string())))?;
    let password_hash = password_hash_string.password_hash();

    let argon2 = Argon2::default();
    let is_valid = argon2.verify_password(password.as_bytes(), &password_hash).is_ok();

    if !is_valid {
        return Err(ApiError::Unauthorized(Some("Invalid password".to_string())));
    }

    generate_token(&jwt_key)
}

pub fn generate_token(jwt_key: &Vec<u8>) -> Result<String, ApiError> {
    let expires = SystemTime::now() + Duration::from_secs(60 * 60 * 24);
    let expires_unix = expires.duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = Claims {
        exp: expires_unix,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&jwt_key),
    ).map_err(|_| ApiError::Other("Failed to generate JWT".to_string()))
}