use crate::{ApiError, AppState};
use argon2::{Argon2, PasswordVerifier};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use argon2::password_hash::PasswordHashString;
use crate::data::enums::UserRole;

pub struct Authenticated {
    pub claims: Claims,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub id: i64,
    pub role: UserRole,
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
        let jwt_key = match settings.get_jwt_key() {
            Ok(k) => k,
            Err(_) => return Outcome::Error((Status::InternalServerError, ())),       
        };
        let decoding_key = DecodingKey::from_secret(&jwt_key);
        let validation = Validation::default();

        let claims = match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(c) => c.claims,
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        };

        Outcome::Success(Authenticated { claims })
    }
}

pub fn verify_password(password_hash_string: &PasswordHashString, password: &str) -> Result<(), ApiError> {
    let password_hash = password_hash_string.password_hash();

    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &password_hash)
        .map_err(|_| ApiError::BadRequest("Passwords do not match.".to_string()))
}

pub fn generate_token(jwt_key: &[u8], user_id: i64, user_role: UserRole) -> Result<String, ApiError> {
    let expires = SystemTime::now() + Duration::from_secs(60 * 60 /* 1 hour */);
    let expires_unix = expires.duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = Claims {
        exp: expires_unix,
        id: user_id,
        role: user_role
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_key),
    ).map_err(|_| ApiError::Other("Failed to generate JWT".to_string()))
}