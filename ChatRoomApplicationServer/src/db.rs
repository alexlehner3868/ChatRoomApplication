use crate::message::{ChatMessage, RoomInfo};
use crate::models::{DbMessage, DbRoom, DbUser};
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use sqlx::{Pool, Postgres};
use std::env;
use uuid::Uuid;

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

// fetch a user by thier id
pub async fn get_user_by_id(pool: &DbPool, id: Uuid) -> Result<DbUser, sqlx::Error> {
    let user = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE id = $1")
        .bind(id)
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

pub async fn add_user_to_room(
    pool: &DbPool,
    room_db_id: uuid::Uuid,
    user_db_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO room_memberships (room_id, user_id) VALUES ($1, $2) ON CONFLICT (room_id, user_id) DO NOTHING")
        .bind(room_db_id)
        .bind(user_db_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_messages_for_room(
    pool: &DbPool,
    room_id: Uuid,
    limit: i64,
) -> Result<Vec<ChatMessage>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            m.id,
            m.content,
            m.created_at as "created_at!",
            r.room_id AS room_name,
            u.user_id AS user_name
        FROM messages m
        JOIN rooms r ON m.room_id = r.id
        JOIN users u ON m.user_id = u.id
        WHERE m.room_id = $1
        ORDER BY m.created_at ASC
        LIMIT $2
        "#,
        room_id,
        limit
    )
    .fetch_all(pool)
    .await?;

    let msgs = rows
        .into_iter()
        .map(|r| ChatMessage {
            room_id: r.room_name, // Use room name instead of UUID
            user_id: r.user_name, // Use username instead of UUID
            message_id: r.id.to_string(),
            content: r.content,
            timestamp: r.created_at.to_rfc3339(),
        })
        .collect();

    Ok(msgs)
}

pub async fn delete_room_by_room_id(pool: &DbPool, room_id: &str) -> Result<(), sqlx::Error> {
    // get room to get its UUID primary key
    let room = sqlx::query!(
        r#"
        SELECT id
        FROM rooms
        WHERE room_id = $1
        "#,
        room_id
    )
    .fetch_optional(pool)
    .await?;

    let Some(room) = room else {
        // If room not found
        return Err(sqlx::Error::RowNotFound);
    };

    // Delete the room by UUID
    // CASCADE removes memberships + messages automatically
    sqlx::query!(
        r#"
        DELETE FROM rooms
        WHERE id = $1
        "#,
        room.id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_all_rooms(pool: &Pool<Postgres>) -> Result<Vec<RoomInfo>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            room_id,
            owner_id,
            created_at
        FROM rooms
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let results = rows
        .into_iter()
        .map(|r| RoomInfo {
            room_id: r.room_id.to_string(),
            owner: r.owner_id.to_string(),
            users_count: 0,
        })
        .collect();

    Ok(results)
}
pub async fn save_message(
    pool: &DbPool,
    room_id: Uuid,
    user_id: Uuid,
    content: &str,
) -> Result<ChatMessage, sqlx::Error> {
    let row = sqlx::query_as::<_, DbMessage>(
        r#"
        INSERT INTO messages (room_id, user_id, content)
        VALUES ($1, $2, $3)
        RETURNING id, room_id, user_id, content, created_at
        "#,
    )
    .bind(room_id)
    .bind(user_id)
    .bind(content)
    .fetch_one(pool)
    .await?;

    Ok(ChatMessage::from(row))
}

pub async fn remove_user_from_room(
    pool: &DbPool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM room_memberships
        WHERE room_id = $1 AND user_id = $2
        "#,
        room_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
