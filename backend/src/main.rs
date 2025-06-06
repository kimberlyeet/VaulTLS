#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::http::{Method, CookieJar, Cookie, SameSite};
use rocket::serde::json::Json;
use rocket::State;
use std::sync::Arc;
use argon2::password_hash::PasswordHashString;
use rocket::response::Redirect;
use rocket::tokio::sync::Mutex;
use rocket_cors::{AllowedOrigins, CorsOptions};
use serde::Serialize;
use cert::create_ca;
use db::CertificateDB;
use settings::Settings;
use crate::local_auth::{generate_token, verify_password, Authenticated};
use crate::cert::{get_pem, save_ca, Certificate};
use crate::data::api::{CallbackQuery, ChangePasswordRequest, CreateCertificateRequest, CreateUserRequest, DownloadResponse, IsSetupResponse, LoginRequest, LoginResponse, SetupRequest};
use crate::data::enums::UserRole;
use crate::data::error::ApiError;
use crate::helper::hash_password_string;
use crate::oidc_auth::OidcAuth;
use crate::settings::FrontendSettings;

mod db;
mod cert;
mod settings;
mod notification;
mod local_auth;
mod oidc_auth;
mod data;
mod helper;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<CertificateDB>>,
    settings: Arc<Mutex<Settings>>,
    oidc: Arc<Mutex<Option<OidcAuth>>>
}

#[derive(Serialize)]
struct User {
    id: i64,
    name: String,
    email: String,
    #[serde(skip_serializing)]
    password_hash: Option<PasswordHashString>,
    #[serde(skip_serializing)]
    oidc_id: Option<String>,
    role: UserRole
}

#[get("/api")]
fn index() -> &'static str {
    "<h1>mTLS Certificates API</h1>"
}

#[get("/api/certificates")]
async fn get_certificates(
    state: &State<AppState>,
    authentication: Authenticated
) -> Result<Json<Vec<Certificate>>, ApiError> {
    let db = state.db.lock().await;
    let user_id= if authentication.claims.role == UserRole::Admin {
            None
        } else {
            Some(authentication.claims.id)
        };
    let certificates = db.get_all_user_cert(user_id)?;
    Ok(Json(certificates))
}

#[post("/api/certificates", format = "json", data = "<payload>")]
async fn create_user_certificate(
    state: &State<AppState>,
    payload: Json<CreateCertificateRequest>,
    authentication: Authenticated
) -> Result<Json<Certificate>, ApiError> {
    if authentication.claims.role != UserRole::Admin { return Err(ApiError::Unauthorized(None)) }

    let db = state.db.lock().await;

    let ca = db.get_current_ca()?;
    let mut user_cert = cert::create_user_cert(&ca, &payload.cert_name, payload.validity_in_years.unwrap_or(1))?;

    let id = db.insert_user_cert(user_cert.clone(), payload.user_id)?;
    user_cert.set_id(id);

    Ok(Json(user_cert))
}

#[get("/api/certificates/ca/download")]
async fn download_ca(
    state: &State<AppState>
) -> Result<DownloadResponse, ApiError> {
    let db = state.db.lock().await;
    let ca = db.get_current_ca()?;
    let pem = get_pem(&ca)?;
    Ok(DownloadResponse::new(pem, "ca_certificate.pem"))
}
#[get("/api/certificates/<id>/download")]
async fn download_certificate(
    state: &State<AppState>,
    id: i64,
    authentication: Authenticated
) -> Result<DownloadResponse, ApiError> {
    let db = state.db.lock().await;
    let (user_id, pkcs12) = db.get_user_pkcs12(id)?;
    if user_id != authentication.claims.id { return Err(ApiError::Unauthorized(None)) }
    Ok(DownloadResponse::new(pkcs12, "user_certificate.p12"))
}

#[delete("/api/certificates/<id>")]
async fn delete_user_cert(
    state: &State<AppState>,
    id: i64,
    authentication: Authenticated
) -> Result<(), ApiError> {
    if authentication.claims.role != UserRole::Admin { return Err(ApiError::Unauthorized(None)) }
    let db = state.db.lock().await;
    db.delete_user_cert(id)?;
    Ok(())
}

#[get("/api/settings")]
async fn fetch_settings(
    state: &State<AppState>,
    authentication: Authenticated
) -> Result<Json<FrontendSettings>, ApiError> {
    if authentication.claims.role != UserRole::Admin { return Err(ApiError::Unauthorized(None)) }
    let settings = state.settings.lock().await;
    let frontend_settings = FrontendSettings(settings.clone());
    Ok(Json(frontend_settings))
}

#[put("/api/settings", format = "json", data = "<payload>")]
async fn update_settings(
    state: &State<AppState>,
    payload: Json<Settings>,
    authentication: Authenticated
) -> Result<(), ApiError> {
    if authentication.claims.role != UserRole::Admin { return Err(ApiError::Unauthorized(None)) }
    let mut settings = state.settings.lock().await;
    let mut oidc = state.oidc.lock().await;

    settings.set_settings(&payload)?;
    
    if let Some(oidc) = &mut *oidc {
        oidc.update_config(settings.get_oidc()).await?;
    }

    Ok(())
}

#[get("/api/is_setup")]
async fn is_setup(
    state: &State<AppState>
) -> Result<Json<IsSetupResponse>, ApiError> {
    let settings = state.settings.lock().await;
    let db = state.db.lock().await;
    let is_setup = db.is_setup();
    let has_password = settings.password_enabled();
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
    let settings = state.settings.lock().await;
    let db = state.db.lock().await;

    if db.is_setup() {
        return Err(ApiError::Unauthorized(None))
    }

    if setup_req.password.is_some() && settings.get_oidc().auth_url.is_empty() {
        return Err(ApiError::Other("Password is required".to_string()))
    }

    let mut user = User{
        id: -1,
        name: setup_req.name.clone(),
        email: setup_req.email.clone(),
        password_hash: hash_password_string(&setup_req.password)?,
        oidc_id: None,
        role: UserRole::Admin,
    };

    db.add_user(&mut user)?;

    let mut ca = create_ca(&setup_req.ca_name, setup_req.ca_validity_in_years)?;
    save_ca(&ca)?;
    let ca_id = db.insert_ca(&ca)?;
    ca.set_id(ca_id);

    Ok(())
}

#[post("/api/auth/login", format = "json", data = "<login_req_opt>")]
async fn login(
    state: &State<AppState>,
    jar: &CookieJar<'_>,
    login_req_opt: Json<LoginRequest>
) -> Result<Json<LoginResponse>, ApiError> {
    if let Some(token_cookie) = jar.get_private("auth_token") {
        let token = token_cookie.value().to_string();
        return Ok(Json(LoginResponse { token }));
    }

    if let Some(password) = &login_req_opt.password {
        let settings = state.settings.lock().await;
        let db = state.db.lock().await;
        let user: User = db.get_user(login_req_opt.user_id)?;
        if let Some(password_hash) = user.password_hash {
            verify_password(&password_hash, password)?;
            let token = generate_token(&settings.get_jwt_key(), user.id, user.role)?;

            return Ok(Json(LoginResponse { token }));
        }
    }

    Err(ApiError::Unauthorized(None))
}

#[post("/api/auth/change_password", data = "<change_pass_req>")]
async fn change_password(
    state: &State<AppState>,
    change_pass_req: Json<ChangePasswordRequest>,
    authentication: Authenticated
) -> Result<(), ApiError> {
    let db = state.db.lock().await;
    let user_id = authentication.claims.id;

    let password_hash = db.get_user(user_id)?.password_hash;

    if let Some(password_hash) = password_hash {
        match &change_pass_req.old_password {
            Some(old_password) => verify_password(&password_hash, old_password)?,
            None => return Err(ApiError::Unauthorized(Some("Password is required".to_string())))
        }
    }

    db.set_user_password(user_id, &change_pass_req.new_password)
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
    let db = state.db.lock().await;

    match &mut *oidc_option {
        Some(oidc) => {
            let mut user = oidc.verify_auth_code(response.code.to_string(), response.state.to_string()).await?;

            db.register_oidc_user(&mut user)?;

            let jwt_key = settings.get_jwt_key();
            let token = generate_token(&jwt_key, user.id, user.role)?;

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

#[get("/api/users")]
async fn get_users(
    state: &State<AppState>,
    authentication: Authenticated
) -> Result<Json<Vec<User>>, ApiError> {
    if authentication.claims.role != UserRole::Admin { return Err(ApiError::Unauthorized(None)) }
    let db = state.db.lock().await;
    let users = db.get_all_user()?;
    Ok(Json(users))
}

#[post("/api/users", format = "json", data = "<payload>")]
async fn create_user(
    state: &State<AppState>,
    payload: Json<CreateUserRequest>,
    authentication: Authenticated
) -> Result<Json<i64>, ApiError> {
    if authentication.claims.role != UserRole::Admin { return Err(ApiError::Unauthorized(None)) }

    let db = state.db.lock().await;

    let mut user = User{
        id: -1,
        name: payload.user_name.to_string(),
        email: payload.user_email.to_string(),
        password_hash: hash_password_string(&payload.password)?,
        oidc_id: None,
        role: payload.role
    };

    db.add_user(&mut user)?;

    Ok(Json(user.id))
}

#[delete("/api/users/<id>")]
async fn delete_user(
    state: &State<AppState>,
    id: i64,
    authentication: Authenticated
) -> Result<(), ApiError> {
    if authentication.claims.role != UserRole::Admin { return Err(ApiError::Unauthorized(None)) }
    let db = state.db.lock().await;
    db.delete_user(id)?;
    Ok(())
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
                download_ca,
                download_certificate,
                delete_user_cert,
                fetch_settings,
                update_settings,
                is_setup,
                setup,
                login,
                change_password,
                oidc_login,
                oidc_callback,
                get_users,
                create_user,
                delete_user
            ],
        )
        .attach(cors.to_cors().unwrap())
        .attach(AdHoc::config::<Settings>())
}
