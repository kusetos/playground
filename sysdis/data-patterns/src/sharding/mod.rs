use std::time::Instant;

use axum::{
    Json,
    extract::{Path, State},
};
use reqwest::StatusCode;
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct AppState {
    db_shards: Vec<PgPool>,
}

pub fn shard_for_key(id: &str) -> usize {
    let hash = id
        .as_bytes()
        .iter()
        .fold(0, |acc, &byte| acc ^ byte as usize);
    if hash % 2 == 0 { 0 } else { 1 }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreateUser {
    id: String,
    name: String,
    email: String,
}
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct User {
    id: String,
    name: String,
    email: String,
}
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let t = Instant::now();
    let shard = shard_for_key(&id);
    let pool = &state.db_shards[shard];
    let result = sqlx::query("SELECT * FROM users WHERE id = $1")
        .bind(&id)
        .fetch_one(pool)
        .await;

    match result {
        Ok(row) => {
            let user = serde_json::json!({ "id": row.get::<String, _>("id"), "name": row.get::<String, _>("name"), "email": row.get::<String, _>("email") });
            tracing::info!("User found: {:?}", user);
            (StatusCode::OK, Json(user))
        }
        Err(e) => {
            tracing::error!("Failed to get user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal Server Error" })),
            )
        }
    }
}
pub async fn get_users(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let t = Instant::now();

    let mut set = tokio::task::JoinSet::new();
    for pool in state.db_shards.clone() {
        set.spawn(async move {
            sqlx::query("SELECT * FROM users")
                .fetch_all(&pool)
                .await
                .unwrap_or_default()
        });
    }

    let mut users = Vec::new();
    while let Some(Ok(rows)) = set.join_next().await {
        for row in rows {
            users.push(serde_json::json!({
                "id": row.get::<String, _>("id"),
                "name": row.get::<String, _>("name"),
                "email": row.get::<String, _>("email"),
            }));
        }
    }

    tracing::info!(
        elapsed_ms = t.elapsed().as_millis(),
        count = users.len(),
        "GET /users"
    );
    (StatusCode::OK, Json(serde_json::json!(users)))
}

pub async fn post_users(
    State(state): State<AppState>,
    Json(body): Json<CreateUser>,
) -> (StatusCode, Json<serde_json::Value>) {
    let t = Instant::now();
    let shard = shard_for_key(&body.id);
    let pool = &state.db_shards[shard];
    let result = sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $2, $3)")
        .bind(&body.id)
        .bind(&body.name)
        .bind(&body.email)
        .execute(pool)
        .await;

    match result {
        Ok(_) => {
            tracing::info!(
                pool = shard,
                elapsed_ms = t.elapsed().as_millis(),
                "POST /users"
            );
            (
                StatusCode::CREATED,
                Json(
                    serde_json::json!({ "id": &body.id, "name": &body.name, "email": &body.email }),
                ),
            )
        }
        Err(e) => {
            tracing::error!(error = %e, id = &body.id, "POST /users failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        routing::{get, post},
    };
    use redis::aio::ConnectionManager;
    use sqlx::PgPool;

    use super::*;
    #[test]
    fn test_shard_for_key() {
        assert_eq!(shard_for_key("user:1"), 0);
        assert_eq!(shard_for_key("user:2"), 1);
        assert_eq!(shard_for_key("user:3"), 0);
        assert_eq!(shard_for_key("user:4"), 1);
    }

    #[tokio::test]
    async fn it_works() {
        tracing_subscriber::fmt().init();
        dotenvy::dotenv().ok();
        let shard_1_url = std::env::var("DATABASE_SHARD_0_URL").unwrap();
        let shard_2_url = std::env::var("DATABASE_SHARD_1_URL").unwrap();

        let pool_1 = PgPool::connect(&shard_1_url).await.unwrap();
        let pool_2 = PgPool::connect(&shard_2_url).await.unwrap();

        let pools = vec![pool_1, pool_2];

        let state = AppState { db_shards: pools };

        let app = Router::new()
            .route("/users", post(post_users))
            .route("/users/{id}", get(get_user))
            .route("/users", get(get_users))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    #[tokio::test]
    async fn test_post_users() {
        let client = reqwest::Client::new();
        for _ in 0..10 {
            let id = uuid::Uuid::new_v4().to_string();
            let post_response = client
                .post("http://localhost:3000/users")
                .json(&CreateUser {
                    id: id.clone(),
                    name: "test".to_string(),
                    email: "test@example.com".to_string(),
                })
                .send()
                .await
                .unwrap();
            assert!(post_response.status().is_success());
            let get_response = client
                .get(&format!("http://localhost:3000/users/{}", id))
                .send()
                .await
                .unwrap();
            assert!(get_response.status().is_success());
        }
    }

    #[tokio::test]
    async fn test_get_users() {
        let client = reqwest::Client::new();
        let get_response = client
            .get("http://localhost:3000/users")
            .send()
            .await
            .unwrap();
        let users: Vec<User> = serde_json::from_str(&get_response.text().await.unwrap()).unwrap();
        println!("len: {}, {:?}", users.len(), users);
        // assert!(get_response.status().is_success());
    }
}
