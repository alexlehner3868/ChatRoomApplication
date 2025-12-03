use crate::db::DbPool;
use crate::message::{AuthSuccessResponse, ErrorResponse, LoginRequest, RegisterRequest};
use crate::AppState;
use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode, Json};
use axum::http::HeaderMap;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header, decode, DecodingKey, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use thiserror::Error;

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

pub async fn register(db: &DbPool, req: RegisterRequest) -> Result<AuthSuccessResponse, AuthError> {
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

    Ok(AuthSuccessResponse {
        token: "".to_string(),
        user_id: req.user_id,
    })
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
        None => return Err(AuthError::BadRequest("User does not exist".into())),
    };

    // verify password
    let parsed_hash =
        PasswordHash::new(&row.password_hash).map_err(|e| AuthError::Hash(e.to_string()))?;
    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::BadRequest("Incorrect password".into()))?;

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
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = req.user_id.clone();
    match register(&state.db_pool, req).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(AuthError::BadRequest(msg)) if msg.contains("already exists") => {
            let err = ErrorResponse::UserAlreadyExists { user_id };
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::ServerError {
                message: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = req.user_id.clone();
    match login(&state.db_pool, req).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(AuthError::BadRequest(msg)) if msg.contains("Incorrect password") => {
            let err = ErrorResponse::InvalidPassword {
                message: "Invalid password".into(),
            };
            (StatusCode::UNAUTHORIZED, Json(err)).into_response()
        }
        Err(AuthError::BadRequest(msg)) if msg.contains("does not exist") => {
            let err = ErrorResponse::UserNotFound { user_id };
            (StatusCode::UNAUTHORIZED, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::ServerError {
                message: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// Extracts and validates JWT from the "Authorization: Bearer <token>" header.
/// Returns Ok(user_id) if valid, otherwise Err(ErrorResponse)
pub async fn authenticate_request(headers: &HeaderMap) -> Result<String, ErrorResponse> {
    // Get Authorization header
    let auth_header = match headers.get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => {
            return Err(ErrorResponse::AuthenticationFailed {
                message: "Missing Authorization header".into(),
            });
        }
    };

    // Expected format: "Bearer <token>"
    if !auth_header.starts_with("Bearer ") {
        return Err(ErrorResponse::AuthenticationFailed {
            message: "Invalid Authorization header format".into(),
        });
    }

    let token = &auth_header[7..]; // strip "Bearer "

    // Load secret
    let secret = match std::env::var("JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            return Err(ErrorResponse::ServerError {
                message: "Server missing JWT_SECRET".into(),
            });
        }
    };

    // Decode and validate token
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data,
        Err(_) => {
            return Err(ErrorResponse::AuthenticationFailed {
                message: "Invalid or expired token".into(),
            });
        }
    };
    // Return user_id from the token's subject (sub)
    Ok(token_data.claims.sub)
}
