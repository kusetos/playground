// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// struct User {
//     id: i32,
//     username: String,
// }

// #[cfg(test)]
// mod tests {
//     use redis::aio::ConnectionManager;
//     use sqlx::PgPool;

//     use super::*;

//     #[tokio::test]
//     async fn it_works() {
//         dotenvy::dotenv().ok();
//         let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//         let redis_url =
//             std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

//         // 2. Connect to PostgreSQL
//         println!("Connecting to PostgreSQL...");
//         let db_pool = PgPool::connect(&database_url).await?;
//         println!("PostgreSQL connected successfully!");

//         // 3. Connect to Redis
//         println!("Connecting to Redis...");
//         let client = redis::Client::open(redis_url)?;
//         let mut redis_conn = ConnectionManager::new(client).await?;
//         println!("Redis connected successfully!");

//         // 4. Test Serde JSON
//         let user = User {
//             id: 1,
//             username: "dauzhant".to_string(),
//         };
//         let json_string = serde_json::to_string(&user)?;
//         println!("Serialized User to JSON: {}", json_string);
//     }
// }
