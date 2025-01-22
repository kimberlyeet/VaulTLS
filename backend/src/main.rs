use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get},
    Json, Router,
};
use cert::create_ca;
use db::CertificateDB;
use hyper::Method;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use settings::Settings;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use thiserror::Error;

mod db;
mod cert;
mod settings;
mod notification;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<CertificateDB>>,
    settings: Arc<Mutex<Settings>>
}

#[derive(Default, Clone)]
struct Certificate {
    id: i64,
    name: String,
    created_on: i64,
    valid_until: i64,
    pkcs12: Vec<u8>,
    cert: Vec<u8>,
    key: Vec<u8>
}

impl Serialize for Certificate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Certificate", 4)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("created_on", &self.created_on)?;
        s.serialize_field("valid_until", &self.valid_until)?;
        s.end()
    }
}

#[derive(Deserialize)]
struct CreateCertificateRequest {
    name: String,
    validity_in_years: Option<u32>,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("OpenSSL error: {0}")]
    OpenSsl(#[from] openssl::error::ErrorStack),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error), // or whatever you commonly throw
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let body = self.to_string();
        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() {
    let db_path = std::path::Path::new("./certificates.db3");
    let db_initialized = db_path.exists();
    let db = CertificateDB::new(db_path).expect("Failed opening SQLite database.");
    if !db_initialized {
        db.initialize_db().expect("Failed initialzing database");
        let ca = create_ca("Hymalia CA").expect("Failed creating CA");
        db.insert_ca(ca).expect("Failed insert CA");
    }

    let settings = Settings::new(None).unwrap();
    settings.save(None).unwrap();

    let app_state = AppState {
        db: Arc::new(Mutex::new(db)),
        settings: Arc::new(Mutex::new(settings))
    };

    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow all origins
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]) // Allow specific HTTP methods
        .allow_headers(Any); // Allow all headers

    let app = Router::new()
        .route("/", get(index))
        .route("/certificates", get(get_certificates).post(create_user_certificate))
        .route("/certificates/:id/download", get(download_certificate))
        .route("/certificates/:id", delete(delete_user_cert))
        .route("/settings", get(fetch_settings).put(update_settings))
        .with_state(app_state)
        .layer(cors);

    println!("Server running on http://127.0.0.1:3737");
    axum::Server::bind(&"127.0.0.1:3737".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html("<h1>mTLS Certificates API</h1>")
}

async fn get_certificates(State(state): State<AppState>) -> impl IntoResponse {
    let db = state.db.lock().await;
    let certificates = db.get_all_user_cert()?;
    Ok::<Json<Vec<Certificate>>, ApiError>(Json(certificates))
}

async fn create_user_certificate(
    State(state): State<AppState>,
    Json(payload): Json<CreateCertificateRequest>,
) -> Result<Json<Certificate>, ApiError> {
    let db = state.db.lock().await;

    let ca = db.get_current_ca()?;

    let mut user_cert =
        cert::create_user_cert(&ca, &payload.name, payload.validity_in_years.unwrap().into())?;

    let id = db.insert_user_cert(user_cert.clone())?;
    user_cert.id = id;

    Ok(Json(user_cert))
}

async fn download_certificate(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let db = state.db.lock().await;
    let pkcs12 = db.get_user_pkcs12(id.parse().unwrap());

    match pkcs12 {
        Ok(pkcs12) => {
            (
                StatusCode::OK,
                [
                    ("Content-Type", "application/pkcs12"),
                    ("Content-Disposition", "attachment; filename=\"certificate.crt\""),
                ],
                pkcs12,
            )
                .into_response()
        }
        _ => (StatusCode::NOT_FOUND, "Certificate not found").into_response(),
    }
}

async fn delete_user_cert(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(), ApiError> {
    let db = state.db.lock().await;
    db.delete_user_cert(id.parse().unwrap())?;
    Ok(())
}

async fn fetch_settings(
    State(state): State<AppState>
) -> Result<Json<Settings>, ApiError> {
    let settings = state.settings.lock().await;
    Ok(Json(settings.clone()))
}

async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<Settings>,
) -> Result<(), ApiError> {
    let mut settings = state.settings.lock().await;
    Ok(settings.set_settings(&payload)?)
}