use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHashString, SaltString};
use serde::Serializer;
use crate::data::error::ApiError;

pub fn hash_password(password: &String) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    Ok(argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|_| ApiError::Other("Failed to hash password".to_string()))?
        .to_string())
}
pub fn hash_password_string(password: &Option<String>) -> Result<Option<PasswordHashString>, ApiError> {
    Ok(match password {
        Some(password) => {
            let password_hash = hash_password(password)?;
            Some(PasswordHashString::new(&*password_hash)?)
        },
        None => None,
    })
}

pub fn serialize_password_hash<S>(password_hash: &Option<PasswordHashString>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_bool(password_hash.is_some())
}
