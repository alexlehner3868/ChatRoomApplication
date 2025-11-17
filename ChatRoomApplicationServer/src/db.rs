use crate::models::DbUser;
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};

// use chrono::Utc;
// use rand::rngs::OsRng;
use sqlx::{Pool, Postgres};
use std::env;
// use uuid::Uuid;

pub type DbPool = Pool<Postgres>;

// Database Connection Functions
pub async fn init_db_from_env() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    init_db(&database_url).await
}
pub async fn init_db(database_url: &str) -> DbPool {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Failed to connect to database")
}

//User-related DB functions
pub async fn create_user(
    pool: &DbPool,
    user_id: &str,
    password: &str,
) -> Result<DbUser, sqlx::Error> {
    // Generate a random salt and hash the password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    let user = sqlx::query_as::<_, DbUser>(
        "INSERT INTO users (user_id, password_hash) VALUES ($1, $2) RETURNING *",
    )
    .bind(user_id)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Fetch a user by their user_id
pub async fn get_user_by_user_id(pool: &DbPool, user_id: &str) -> Result<DbUser, sqlx::Error> {
    let user = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(user)
}
