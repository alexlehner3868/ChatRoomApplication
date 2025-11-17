use crate::db::DbPool;
use crate::message::{AuthSuccessResponse, LoginRequest, RegisterRequest};
use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header}; //{ decode, DecodingKey, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use crate::state::GlobalState;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("hash error: {0}")]
    Hash(String),
    #[error("{0}")]
    BadRequest(String),
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn register(db: &DbPool, req: RegisterRequest) -> Result<(), AuthError> {
    // check if user exists
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1) as \"exists!\"",
        req.user_id
    )
    .fetch_one(db)
    .await?;

    if exists {
        return Err(AuthError::BadRequest(format!(
            "User {} already exists",
            req.user_id
        )));
    }

    // hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AuthError::Hash(e.to_string()))?
        .to_string();

    sqlx::query!(
        r#"
        INSERT INTO users (user_id, password_hash)
        VALUES ($1, $2)
        "#,
        req.user_id,
        password_hash
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn login(db: &DbPool, req: LoginRequest) -> Result<AuthSuccessResponse, AuthError> {
    // fetch user row
    let row = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE user_id = $1",
        req.user_id
    )
    .fetch_optional(db)
    .await?;

    let row = match row {
        Some(r) => r,
        None => return Err(AuthError::BadRequest("Invalid credentials".into())),
    };

    // verify password
    let parsed_hash =
        PasswordHash::new(&row.password_hash).map_err(|e| AuthError::Hash(e.to_string()))?;
    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::BadRequest("Invalid credentials".into()))?;

    // create JWT
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: req.user_id.clone(),
        exp: expiration.timestamp() as usize,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError::BadRequest(format!("JWT encode error: {}", e)))?;

    Ok(AuthSuccessResponse {
        token,
        user_id: req.user_id,
    })
}

pub async fn register_handler(
    State(state): State<GlobalState>,
    Json(req): Json<RegisterRequest>,
) -> impl axum::response::IntoResponse {
    // tracing::info!("Received Register request: {:?}", req);
    match register(&state.db_pool, req).await {
        Ok(_) => (StatusCode::CREATED, "User registered").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn login_handler(
    State(state): State<GlobalState>,
    Json(req): Json<LoginRequest>,
) -> impl axum::response::IntoResponse {
    match login(&state.db_pool, req).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (StatusCode::UNAUTHORIZED, e.to_string()).into_response(),
    }
}
