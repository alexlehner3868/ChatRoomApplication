// use crate::models::DbUser;
// use argon2::password_hash::rand_core::OsRng;
// use argon2::{
//     password_hash::{PasswordHasher, SaltString},
//     Argon2,
// };
use sqlx::{Pool, Postgres};
use std::env;

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
