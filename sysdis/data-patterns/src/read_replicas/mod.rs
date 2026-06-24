use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use std::time::Instant;
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Item {
    id: String,
    value: Option<String>,
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
    replica_db: PgPool,
}
#[cfg(test)]
mod tests {
    use reqwest::Client;
    use sqlx::PgPool;
    use uuid::Uuid;
    #[tokio::test]
    async fn test_read_replica() {
        tracing_subscriber::fmt().init();
        let client = Client::new();
        let n = 100;
        let mut found = 0;
        let mut not_found = 0;

        for i in 0..n {
            let id = Uuid::new_v4().to_string();

            // POST to primary
            client
                .post("http://localhost:3000/items")
                .json(&serde_json::json!({ "id": &id, "value": "probe" }))
                .send()
                .await
                .unwrap();

            // Immediately GET — force primary to bypass replication lag
            let items: Vec<Item> = client
                .get("http://localhost:3000/items")
                .header("x-read-primary", "true")
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let visible = items.iter().any(|item| item.id == id);
            if visible {
                found += 1;
                tracing::info!(iter = i, id, result = "found");
            } else {
                not_found += 1;
                tracing::warn!(iter = i, id, result = "NOT FOUND (replication lag)");
            }
        }

        tracing::info!(n, found, not_found, "experiment complete");
        println!("\n=== results: {found}/{n} visible immediately, {not_found} lagged ===");
        assert_eq!(not_found, 0, "primary override should have zero lag");
    }
    #[tokio::test]
    async fn read_replica_lagged() {
        tracing_subscriber::fmt().init();
        let client = Client::new();
        let n = 20;
        let mut found = 0;
        let mut not_found = 0;

        for i in 0..n {
            let id = Uuid::new_v4().to_string();

            // POST to primary
            client
                .post("http://localhost:3000/items")
                .json(&serde_json::json!({ "id": &id, "value": "probe" }))
                .send()
                .await
                .unwrap();

            // Immediately GET from replica — no override header
            let items: Vec<Item> = client
                .get("http://localhost:3000/items")
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let visible = items.iter().any(|item| item.id == id);
            if visible {
                found += 1;
                tracing::info!(iter = i, id, result = "found");
            } else {
                not_found += 1;
                tracing::warn!(iter = i, id, result = "NOT FOUND (replication lag)");
            }
        }

        println!("\n=== lagged results: {found}/{n} visible, {not_found} lagged ===");
    }

    use super::*;

    #[tokio::test]
    async fn read_replica() {
        tracing_subscriber::fmt().init();
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let replica_url = std::env::var("DATABASE_REPLICA_URL").unwrap();

        let pool = PgPool::connect(&database_url).await.unwrap();
        let rep_pool = PgPool::connect(&replica_url).await.unwrap();

        let state = AppState {
            db: pool,
            replica_db: rep_pool,
        };

        let app = Router::new()
            .route("/items", get(get_items))
            .route("/items", post(post_items))
            .route("/debug/replication", get(replication_lag))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
        println!("");
    }
}
async fn get_items(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Vec<Item>>) {
    let t = Instant::now();
    let use_primary = headers.get("x-read-primary").is_some();
    let pool = if use_primary {
        &state.db
    } else {
        &state.replica_db
    };
    let pool_name = if use_primary { "primary" } else { "replica" };

    let items = sqlx::query_as::<_, Item>("SELECT id, value FROM items")
        .fetch_all(pool)
        .await
        .unwrap();
    tracing::info!(
        pool = pool_name,
        rows = items.len(),
        elapsed_ms = t.elapsed().as_millis(),
        "GET /items"
    );

    (StatusCode::OK, Json(items))
}
async fn post_items(
    State(state): State<AppState>,
    Json(body): Json<CreateItem>,
) -> (StatusCode, Json<serde_json::Value>) {
    let t = Instant::now();
    let result = sqlx::query("INSERT INTO items (id, value) VALUES ($1, $2)")
        .bind(&body.id)
        .bind(&body.value)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            tracing::info!(
                pool = "primary",
                elapsed_ms = t.elapsed().as_millis(),
                "POST /items"
            );
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "id": &body.id, "value": &body.value })),
            )
        }
        Err(e) => {
            tracing::error!(error = %e, id = &body.id, "POST /items failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    }
}

async fn replication_lag(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    // pg_stat_replication lives on the primary — one row per connected standby
    let primary_rows = sqlx::query(
        r#"
        SELECT
            client_addr::text                                    AS client_addr,
            state,
            (EXTRACT(EPOCH FROM write_lag)  * 1000.0)::float8   AS write_lag_ms,
            (EXTRACT(EPOCH FROM flush_lag)  * 1000.0)::float8   AS flush_lag_ms,
            (EXTRACT(EPOCH FROM replay_lag) * 1000.0)::float8   AS replay_lag_ms
        FROM pg_stat_replication
        "#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    // pg_last_xact_replay_timestamp on the replica tells us how stale it is
    let replica_lag_ms: Option<f64> = sqlx::query_scalar(
        "SELECT (EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp())) * 1000.0)::float8",
    )
    .fetch_one(&state.replica_db)
    .await
    .unwrap();

    use sqlx::Row;
    let standbys: Vec<_> = primary_rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "client_addr":   r.get::<Option<String>, _>("client_addr"),
                "state":         r.get::<Option<String>, _>("state"),
                "write_lag_ms":  r.get::<Option<f64>, _>("write_lag_ms"),
                "flush_lag_ms":  r.get::<Option<f64>, _>("flush_lag_ms"),
                "replay_lag_ms": r.get::<Option<f64>, _>("replay_lag_ms"),
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "primary_sees": standbys,
            "replica_behind_ms": replica_lag_ms,
        })),
    )
}

#[derive(Deserialize)]
struct CreateItem {
    id: String,
    value: String,
}
