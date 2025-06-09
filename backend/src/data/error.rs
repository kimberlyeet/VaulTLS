use std::fmt::Display;
use rocket::http::Status;
use rocket::Request;
use rocket::response::status::Custom;

#[derive(Debug)]
pub enum ApiError {
    Database(rusqlite::Error),
    OpenSsl(openssl::error::ErrorStack),
    Unauthorized(Option<String>),
    BadRequest(String),
    Forbidden(Option<String>),
    Other(String),
}

impl<'r> rocket::response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            ApiError::Database(e) => Custom(Status::InternalServerError, e.to_string()).respond_to(req),
            ApiError::OpenSsl(e) => Custom(Status::InternalServerError, e.to_string()).respond_to(req),
            ApiError::Unauthorized(e) => Custom(Status::Unauthorized, e.unwrap_or(Default::default()).to_string()).respond_to(req),
            ApiError::BadRequest(e) => Custom(Status::BadRequest, e).respond_to(req),
            ApiError::Forbidden(e) => Custom(Status::Forbidden, e).respond_to(req),
            ApiError::Other(e) => Custom(Status::InternalServerError, e).respond_to(req),
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(error: rusqlite::Error) -> Self {
        ApiError::Database(error)
    }
}

impl From<openssl::error::ErrorStack> for ApiError {
    fn from(error: openssl::error::ErrorStack) -> Self {
        ApiError::OpenSsl(error)
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(error: argon2::password_hash::Error) -> Self {
        ApiError::Unauthorized(Some(error.to_string()))
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        ApiError::Other(error.to_string())
    }
}