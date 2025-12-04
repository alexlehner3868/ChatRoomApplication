use crate::message::ChatMessage;
use crate::models::{DbMessage, DbRoom, DbUser};
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
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

// Fetch a user by their user_id
pub async fn get_user_by_user_id(pool: &DbPool, user_id: &str) -> Result<DbUser, sqlx::Error> {
    let user = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

// Room related DB functions
pub async fn create_room(
    pool: &DbPool,
    room_id: &str,
    room_password: &str,
    owner_uuid: uuid::Uuid,
) -> Result<DbRoom, sqlx::Error> {
    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(room_password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let room = sqlx::query_as::<_, DbRoom>(
        r#"
        INSERT INTO rooms (room_id, room_password_hash, owner_id)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(room_id)
    .bind(password_hash)
    .bind(owner_uuid)
    .fetch_one(pool)
    .await?;

    Ok(room)
}

pub async fn get_room_by_room_id(pool: &DbPool, room_id: &str) -> Result<DbRoom, sqlx::Error> {
    let room = sqlx::query_as::<_, DbRoom>("SELECT * FROM rooms WHERE room_id = $1")
        .bind(room_id)
        .fetch_one(pool)
        .await?;

    Ok(room)
}
pub async fn verify_room_password(
    stored_hash: &str,
    provided_password: &str,
) -> Result<bool, argon2::password_hash::Error> {
    use argon2::PasswordVerifier;

    let parsed_hash = argon2::PasswordHash::new(stored_hash)?;
    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(provided_password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub async fn add_user_to_room(
    pool: &DbPool,
    room_db_id: uuid::Uuid,
    user_db_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO room_memberships (room_id, user_id) VALUES ($1, $2)")
        .bind(room_db_id)
        .bind(user_db_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_messages_for_room(
    pool: &DbPool,
    room_id: uuid::Uuid,
    limit: i64,
) -> Result<Vec<ChatMessage>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            room_id,
            user_id,
            content,
            created_at as "created_at!"
        FROM messages
        WHERE room_id = $1
        ORDER BY created_at ASC
        LIMIT $2
        "#,
        room_id,
        limit,
    )
    .fetch_all(pool)
    .await?;

    let msgs = rows
        .into_iter()
        .map(|r| ChatMessage {
            room_id: r.room_id.to_string(),
            user_id: r.user_id.to_string(),
            message_id: r.id.to_string(),
            content: r.content,
            timestamp: r.created_at.to_rfc3339(),
        })
        .collect();

    Ok(msgs)
}
