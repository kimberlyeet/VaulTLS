#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::http::{ContentType, Status, Method};
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use std::sync::Arc;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket_cors::{AllowedOrigins, CorsOptions};

mod db;
mod cert;
mod settings;
mod notification;
mod auth;

use cert::create_ca;
use db::CertificateDB;
use settings::Settings;
use crate::auth::{verify_password, Authenticated};
use crate::cert::Certificate;
use crate::settings::FrontendSettings;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<CertificateDB>>,
    settings: Arc<Mutex<Settings>>,
}

#[derive(Deserialize)]
struct CreateCertificateRequest {
    name: String,
    validity_in_years: Option<u64>,
}

#[derive(Debug)]
enum ApiError {
    Database(rusqlite::Error),
    OpenSsl(openssl::error::ErrorStack),
    Unauthorized(Option<String>),
    Other(String),
}

impl<'r> rocket::response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self {
            ApiError::Database(e) => Custom(Status::InternalServerError, e.to_string()).respond_to(req),
            ApiError::OpenSsl(e) => Custom(Status::InternalServerError, e.to_string()).respond_to(req),
            ApiError::Unauthorized(e) => Custom(Status::Unauthorized, e.unwrap_or(Default::default()).to_string()).respond_to(req),
            ApiError::Other(e) => Custom(Status::InternalServerError, e).respond_to(req),
        }
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

#[derive(Deserialize)]
struct SetupRequest {
    name: String,
    ca_name: String,
    ca_validity_in_years: u64,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[get("/api")]
fn index() -> &'static str {
    "<h1>mTLS Certificates API</h1>"
}

#[get("/api/certificates")]
async fn get_certificates(
    state: &State<AppState>,
    _authenticated: Authenticated
) -> Result<Json<Vec<Certificate>>, ApiError> {
    let db = state.db.lock().await;
    let certificates = db.get_all_user_cert()?;
    Ok(Json(certificates))
}

#[post("/api/certificates", format = "json", data = "<payload>")]
async fn create_user_certificate(
    state: &State<AppState>,
    payload: Json<CreateCertificateRequest>,
    _authenticated: Authenticated
) -> Result<Json<Certificate>, ApiError> {
    let db = state.db.lock().await;

    let ca = db.get_current_ca()?;
    let mut user_cert = cert::create_user_cert(&ca, &payload.name, payload.validity_in_years.unwrap_or(1))?;

    let id = db.insert_user_cert(user_cert.clone())?;
    user_cert.set_id(id);

    Ok(Json(user_cert))
}

#[get("/api/certificates/<id>/download")]
async fn download_certificate(
    state: &State<AppState>,
    id: i64,
    _authenticated: Authenticated
) -> Result<(Status, (ContentType, Vec<u8>)), ApiError> {
    let db = state.db.lock().await;
    let pkcs12 = db.get_user_pkcs12(id)?;
    Ok((Status::Ok, (ContentType::new("application", "pkcs12"), pkcs12)))
}

#[delete("/api/certificates/<id>")]
async fn delete_user_cert(
    state: &State<AppState>,
    id: i64,
    _authenticated: Authenticated
) -> Result<(), ApiError> {
    let db = state.db.lock().await;
    db.delete_user_cert(id)?;
    Ok(())
}

#[get("/api/settings")]
async fn fetch_settings(
    state: &State<AppState>,
    _authenticated: Authenticated
) -> Result<Json<FrontendSettings>, ApiError> {
    let settings = state.settings.lock().await;
    let frontend_settings = FrontendSettings(settings.clone());
    Ok(Json(frontend_settings))
}

#[put("/api/settings", format = "json", data = "<payload>")]
async fn update_settings(
    state: &State<AppState>,
    payload: Json<Settings>,
    _authenticated: Authenticated
) -> Result<(), ApiError> {
    let mut settings = state.settings.lock().await;
    settings.set_settings(&payload)?;
    Ok(())
}

#[get("/api/is_setup")]
async fn is_setup(
    state: &State<AppState>
) -> Result<Json<bool>, ApiError> {
    let settings = state.settings.lock().await;
    Ok(Json(settings.is_setup()))
}

#[post("/api/setup", format = "json", data = "<setup_req>")]
async fn setup(
    state: &State<AppState>,
    setup_req: Json<SetupRequest>
) -> Result<(), ApiError> {
    let mut settings = state.settings.lock().await;
    if settings.is_setup() { return Err(ApiError::Unauthorized(None)) }

    settings.set_username(&setup_req.name)?;
    settings.set_password(&setup_req.password)?;

    let db = state.db.lock().await;
    let ca = create_ca(&setup_req.ca_name, setup_req.ca_validity_in_years)?;
    db.insert_ca(ca)?;

    Ok(())
}

#[post("/api/auth/login", format = "json", data = "<login_req>")]
async fn login(
    state: &State<AppState>,
    login_req: Json<LoginRequest>
) -> Result<Json<LoginResponse>, ApiError> {
    let settings = state.settings.lock().await;
    let token = verify_password(&settings, &login_req.password)?;

    Ok(Json(LoginResponse { token }))
}

#[launch]
fn rocket() -> _ {
    let db_path = std::path::Path::new("./certificates.db3");
    let db_initialized = db_path.exists();
    let db = CertificateDB::new(db_path).expect("Failed opening SQLite database.");
    if !db_initialized {
        db.initialize_db().expect("Failed initializing database");
    }

    let settings = Settings::new(None).unwrap();
    settings.save(None).unwrap();

    let app_state = AppState {
        db: Arc::new(Mutex::new(db)),
        settings: Arc::new(Mutex::new(settings)),
    };

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Put, Method::Delete]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 3737)))
        .manage(app_state)
        .mount(
            "/",
            routes![
                index,
                get_certificates,
                create_user_certificate,
                download_certificate,
                delete_user_cert,
                fetch_settings,
                update_settings,
                is_setup,
                setup,
                login
            ],
        )
        .attach(cors.to_cors().unwrap())
        .attach(AdHoc::config::<Settings>())
}
