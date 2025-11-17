use chat_room_application_server::db::init_db_from_env;
use chat_room_application_server::db::{create_user, get_user_by_user_id};
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    // Load environment variables from .env
    dotenv().ok();
    let pool = init_db_from_env().await;
    println!("Creating test user...");
    let user = create_user(&pool, "testuser", "password123").await.unwrap();
    println!("User created: {:?}", user);

    let fetched_user = get_user_by_user_id(&pool, "testuser").await.unwrap();
    println!("Fetched user: {:?}", fetched_user);
    // println!("🔌 Attempting to connect to Supabase Postgres...");

    // let pool = init_db_from_env().await;

    // // Run a trivial SQL query to confirm the connection
    // let row: (i32,) = sqlx::query_as("SELECT 1")
    //     .fetch_one(&pool)
    //     .await
    //     .expect("❌ Query failed");

    // println!("✅ Success! Database responded with: {:?}", row);
}
