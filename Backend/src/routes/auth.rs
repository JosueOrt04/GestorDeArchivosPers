use actix_web::{post, web, HttpResponse};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use mongodb::{options::IndexOptions, IndexModel};
use validator::Validate;

use crate::{
    config::AppConfig,
    db::AppState,
    errors::ApiError,
    middleware::auth::AuthUser,
    models::user::{AuthResponse, LoginDto, RegisterDto, User},
    utils::{jwt, password},
};

fn users_collection(state: &AppState) -> mongodb::Collection<User> {
    state.db.collection::<User>("users")
}

// Crea índice único para email (si falla, se loggea pero NO rompe)
async fn ensure_email_unique_index(state: &AppState) {
    let col: mongodb::Collection<User> = users_collection(state);

    let options = IndexOptions::builder()
        .unique(true)
        .name(Some("unique_email".to_string()))
        .build();

    let model = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(options)
        .build();

    if let Err(e) = col.create_index(model, None).await {
        eprintln!("Mongo create_index error (unique_email): {:?}", e);
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(register).service(login).service(me);
}

#[post("/register")]
async fn register(
    cfg: web::Data<AppConfig>,
    state: web::Data<AppState>,
    body: web::Json<RegisterDto>,
) -> Result<HttpResponse, ApiError> {
    ensure_email_unique_index(&state).await;

    let mut dto = body.into_inner();

    // Normalizar
    dto.email = dto.email.trim().to_lowercase();
    dto.name = dto.name.trim().to_string();
    dto.role = dto.role.trim().to_string();

    dto.validate()
        .map_err(|e: validator::ValidationErrors| ApiError::BadRequest(e.to_string()))?;

    if dto.role != "cliente" && dto.role != "colaborador" {
        return Err(ApiError::BadRequest(
            "role must be 'cliente' or 'colaborador'".into(),
        ));
    }

    let col: mongodb::Collection<User> = users_collection(&state);

    // Revisar si existe
    let existing = col
        .find_one(doc! { "email": &dto.email }, None)
        .await
        .map_err(|e| {
            eprintln!("Mongo find_one error (register): {:?}", e);
            ApiError::Internal
        })?;

    if existing.is_some() {
        return Err(ApiError::BadRequest("Email already registered".into()));
    }

    let hash = password::hash_password(&dto.password).map_err(ApiError::BadRequest)?;

    let user = User {
        id: ObjectId::new(),
        name: dto.name,
        email: dto.email,
        role: dto.role,
        // ✅ ahora es Option
        password_hash: Some(hash),
        created_at: Utc::now(),
    };

    col.insert_one(&user, None)
        .await
        .map_err(|e| {
            eprintln!("Mongo insert_one error (register): {:?}", e);
            ApiError::Internal
        })?;

    let token = jwt::sign_jwt(
        &user.id.to_hex(),
        &user.email,
        &user.role,
        &cfg.jwt_secret,
        cfg.jwt_exp_minutes,
    )
    .map_err(ApiError::BadRequest)?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: user.clone().into(),
    }))
}

#[post("/login")]
async fn login(
    cfg: web::Data<AppConfig>,
    state: web::Data<AppState>,
    body: web::Json<LoginDto>,
) -> Result<HttpResponse, ApiError> {
    let mut dto = body.into_inner();
    dto.email = dto.email.trim().to_lowercase();

    dto.validate()
        .map_err(|e: validator::ValidationErrors| ApiError::BadRequest(e.to_string()))?;

    let col: mongodb::Collection<User> = users_collection(&state);

    let user = col
        .find_one(doc! { "email": &dto.email }, None)
        .await
        .map_err(|e| {
            eprintln!("Mongo find_one error (login): {:?}", e);
            ApiError::Internal
        })?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".into()))?;

    // ✅ si por alguna razón el usuario no tiene password_hash, no truena
    let hash = user
        .password_hash
        .as_deref()
        .ok_or_else(|| ApiError::Unauthorized("User has no password set".into()))?;

    let ok = password::verify_password(&dto.password, hash)
        .map_err(ApiError::BadRequest)?;

    if !ok {
        return Err(ApiError::Unauthorized("Invalid credentials".into()));
    }

    let token = jwt::sign_jwt(
        &user.id.to_hex(),
        &user.email,
        &user.role,
        &cfg.jwt_secret,
        cfg.jwt_exp_minutes,
    )
    .map_err(ApiError::BadRequest)?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.into(),
    }))
}

#[post("/me")]
async fn me(user: AuthUser) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_id": user.user_id,
        "email": user.email,
        "role": user.role
    })))
}
