#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::http::{ContentType, Status, Method};
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use rocket::tokio::sync::Mutex;
use rocket_cors::{AllowedOrigins, CorsOptions};

mod db;
mod cert;
mod settings;
mod notification;

use cert::create_ca;
use db::CertificateDB;
use settings::Settings;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<CertificateDB>>,
    settings: Arc<Mutex<Settings>>,
}

#[derive(Default, Clone, Serialize)]
struct Certificate {
    id: i64,
    name: String,
    created_on: i64,
    valid_until: i64,
    #[serde(skip)]
    pkcs12: Vec<u8>,
    #[serde(skip)]
    cert: Vec<u8>,
    #[serde(skip)]
    key: Vec<u8>,
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
    Other(String),
}

impl<'r> rocket::response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self {
            ApiError::Database(e) => Custom(Status::InternalServerError, e.to_string()).respond_to(req),
            ApiError::OpenSsl(e) => Custom(Status::InternalServerError, e.to_string()).respond_to(req),
            ApiError::Other(e) => Custom(Status::InternalServerError, e).respond_to(req),
        }
    }
}

#[get("/api")]
fn index() -> &'static str {
    "<h1>mTLS Certificates API</h1>"
}

#[get("/api/certificates")]
async fn get_certificates(state: &State<AppState>) -> Result<Json<Vec<Certificate>>, ApiError> {
    let db = state.db.lock().await;
    let certificates = db.get_all_user_cert().map_err(ApiError::Database)?;
    Ok(Json(certificates))
}

#[post("/api/certificates", format = "json", data = "<payload>")]
async fn create_user_certificate(
    state: &State<AppState>,
    payload: Json<CreateCertificateRequest>,
) -> Result<Json<Certificate>, ApiError> {
    let db = state.db.lock().await;

    let ca = db.get_current_ca().map_err(ApiError::Database)?;
    let mut user_cert = cert::create_user_cert(&ca, &payload.name, payload.validity_in_years.unwrap_or(1))
        .map_err(ApiError::OpenSsl)?;

    let id = db.insert_user_cert(user_cert.clone()).map_err(ApiError::Database)?;
    user_cert.id = id;

    Ok(Json(user_cert))
}

#[get("/api/certificates/<id>/download")]
async fn download_certificate(
    state: &State<AppState>,
    id: String
) -> Result<(Status, (ContentType, Vec<u8>)), ApiError> {
    let db = state.db.lock().await;
    match db.get_user_pkcs12(id.parse::<i64>().unwrap()).map_err(ApiError::Database) {
        Ok(pkcs12) => Ok((Status::Ok, (ContentType::new("application", "pkcs12") ,pkcs12))),
        Err(_) => Err(ApiError::Other("Certificate not found".to_string())),
    }
}

#[delete("/api/certificates/<id>")]
async fn delete_user_cert(
    state: &State<AppState>,
    id: String
) -> Result<(), ApiError> {
    let db = state.db.lock().await;
    db.delete_user_cert(id.parse::<i64>().unwrap())
        .map_err(ApiError::Database)?;
    Ok(())
}

#[get("/api/settings")]
async fn fetch_settings(
    state: &State<AppState>
) -> Result<Json<Settings>, ApiError> {
    let settings = state.settings.lock().await;
    Ok(Json(settings.clone()))
}

#[put("/api/settings", format = "json", data = "<payload>")]
async fn update_settings(
    state: &State<AppState>,
    payload: Json<Settings>
) -> Result<(), ApiError> {
    let mut settings = state.settings.lock().await;
    settings.set_settings(&payload).map_err(|e| ApiError::Other(e.to_string()))?;
    Ok(())
}

#[launch]
fn rocket() -> _ {
    let db_path = std::path::Path::new("./certificates.db3");
    let db_initialized = db_path.exists();
    let db = CertificateDB::new(db_path).expect("Failed opening SQLite database.");
    if !db_initialized {
        db.initialize_db().expect("Failed initializing database");
        let ca = create_ca("Hymalia CA").expect("Failed creating CA");
        db.insert_ca(ca).expect("Failed inserting CA");
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
                update_settings
            ],
        )
        .attach(cors.to_cors().unwrap())
        .attach(AdHoc::config::<Settings>())
}
