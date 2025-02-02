#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::http::{ContentType, Status, Method, CookieJar, Cookie, SameSite};
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use std::sync::Arc;
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket_cors::{AllowedOrigins, CorsOptions};
use cert::create_ca;
use db::CertificateDB;
use settings::Settings;
use crate::local_auth::{generate_token, verify_password, Authenticated};
use crate::cert::{save_ca, Certificate};
use crate::oidc_auth::OidcAuth;
use crate::settings::FrontendSettings;

mod db;
mod cert;
mod settings;
mod notification;
mod local_auth;
mod oidc_auth;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<CertificateDB>>,
    settings: Arc<Mutex<Settings>>,
    oidc: Arc<Mutex<Option<OidcAuth>>>
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

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        ApiError::Other(error.to_string())
    }
}

#[derive(Serialize)]
struct IsSetupResponse {
    setup: bool,
    password: bool,
    oidc: String
}

#[derive(Deserialize)]
struct SetupRequest {
    name: String,
    ca_name: String,
    ca_validity_in_years: u64,
    password: Option<String>,
}

#[derive(Deserialize)]
struct LoginRequest {
    password: Option<String>,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(FromForm)]
struct CallbackQuery {
    code: String,
    state: String
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
) -> Result<Json<IsSetupResponse>, ApiError> {
    let settings = state.settings.lock().await;
    let is_setup = settings.is_setup();
    let has_password = settings.has_password();
    let oidc_url = settings.get_oidc().auth_url.clone();
    Ok(Json(IsSetupResponse {
        setup: is_setup,
        password: has_password,
        oidc: oidc_url
    }))
}

#[post("/api/setup", format = "json", data = "<setup_req>")]
async fn setup(
    state: &State<AppState>,
    setup_req: Json<SetupRequest>
) -> Result<(), ApiError> {
    let mut settings = state.settings.lock().await;
    if settings.is_setup() { return Err(ApiError::Unauthorized(None)) }

    settings.set_username(&setup_req.name)?;

    if setup_req.password.is_some() {
        settings.set_password(&setup_req.password.clone().unwrap())?;
    } else if settings.get_oidc().auth_url.is_empty() {
        return Err(ApiError::Other("Password is required".to_string()))
    }

    let db = state.db.lock().await;
    let ca = create_ca(&setup_req.ca_name, setup_req.ca_validity_in_years)?;
    save_ca(&ca)?;
    db.insert_ca(ca)?;
    Ok(())
}

#[post("/api/auth/login", format = "json", data = "<login_req_opt>")]
async fn login(
    state: &State<AppState>,
    jar: &CookieJar<'_>,
    login_req_opt: Json<LoginRequest>
) -> Result<Json<LoginResponse>, ApiError> {

    if let Some(cookie) = jar.get_private("auth_token") {
        let token = cookie.value().to_string();
        return Ok(Json(LoginResponse { token }));
    }

    if let Some(password) = &login_req_opt.password {
        let settings = state.settings.lock().await;
        let token = verify_password(&settings, password)?;

        return Ok(Json(LoginResponse { token }));
    }

    Err(ApiError::Unauthorized(None))
}

#[get("/api/auth/oidc/login")]
async fn oidc_login(
    state: &State<AppState>,
) -> Result<Redirect, ApiError> {
    let mut oidc_option = state.oidc.lock().await;

    match &mut *oidc_option {
        Some(oidc) => {
            let url = oidc.generate_oidc_url().await?;
            Ok(Redirect::to(url.to_string()))

        }
        None => { Err(ApiError::Unauthorized(Some("OIDC not configured".to_string()))) },
    }
}

#[get("/api/auth/oidc/callback?<response..>")]
async fn oidc_callback(
    state: &State<AppState>,
    jar: &CookieJar<'_>,
    response: CallbackQuery
) -> Result<Redirect, ApiError> {
    let mut oidc_option = state.oidc.lock().await;
    let settings = state.settings.lock().await;

    // or use in match, notice &mut borrowing
    match &mut *oidc_option {
        Some(oidc) => {
            oidc.verify_auth_code(response.code.to_string(), response.state.to_string()).await?;
            let jwt_key = settings.get_jwt_key();
            let token = generate_token(&jwt_key)?;

            let auth_cookie = Cookie::build(("auth_token", token))
                .secure(true)
                .http_only(true)
                .same_site(SameSite::Lax);
            jar.add_private(auth_cookie);

            Ok(Redirect::to("/overview?oidc=success"))
        }
        None => { Err(ApiError::Unauthorized(Some("OIDC not configured".to_string()))) },
    }
}

#[launch]
async fn rocket() -> _ {
    let db_path = std::path::Path::new("./certificates.db3");
    let db_initialized = db_path.exists();
    let db = CertificateDB::new(db_path).expect("Failed opening SQLite database.");
    if !db_initialized {
        db.initialize_db().expect("Failed initializing database");
    }

    let settings = Settings::new(None).unwrap();
    settings.save(None).unwrap();

    let oidc = OidcAuth::new(settings.get_oidc()).await.ok();

    let app_state = AppState {
        db: Arc::new(Mutex::new(db)),
        settings: Arc::new(Mutex::new(settings)),
        oidc: Arc::new(Mutex::new(oidc)),
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
                login,
                oidc_login,
                oidc_callback
            ],
        )
        .attach(cors.to_cors().unwrap())
        .attach(AdHoc::config::<Settings>())
}
