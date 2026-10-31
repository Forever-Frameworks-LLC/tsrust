// Independent Rust server: Axum, serde and a one-connection async SQLx SQLite pool.
use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
#[derive(Deserialize, Serialize)]
struct Payload {
    id: i64,
    label: String,
    value: i64,
}
#[derive(Deserialize)]
struct Lookup {
    id: i64,
}
fn identifier(id: i64) -> bool {
    (1..=1_000_000).contains(&id)
}
fn json_response(payload: Payload) -> Response {
    (
        [("content-type", "application/json")],
        serde_json::to_vec(&payload).unwrap(),
    )
        .into_response()
}
async fn text() -> &'static str {
    "héllo 🦀"
}
async fn json() -> Response {
    json_response(Payload {
        id: 7,
        label: "héllo 🦀".to_owned(),
        value: 42,
    })
}
async fn parse(body: Bytes) -> Response {
    let Ok(input) = serde_json::from_slice::<Payload>(&body) else {
        return (StatusCode::BAD_REQUEST, "invalid").into_response();
    };
    if !identifier(input.id) || !(0..=1_000_000).contains(&input.value) {
        return (StatusCode::BAD_REQUEST, "invalid").into_response();
    }
    json_response(Payload {
        id: input.id,
        label: input.label + "!",
        value: input.value.checked_add(1).unwrap(),
    })
    .into_response()
}
async fn sqlite(State(pool): State<SqlitePool>, body: Bytes) -> Response {
    let Ok(input) = serde_json::from_slice::<Lookup>(&body) else {
        return (StatusCode::BAD_REQUEST, "invalid").into_response();
    };
    if !identifier(input.id) {
        return (StatusCode::BAD_REQUEST, "invalid").into_response();
    }
    let row = sqlx::query("SELECT id, label, value FROM items WHERE id = ?")
        .bind(input.id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    let Some(row) = row else {
        return (StatusCode::NOT_FOUND, "missing").into_response();
    };
    json_response(Payload {
        id: row.try_get("id").unwrap(),
        label: row.try_get("label").unwrap(),
        value: row.try_get("value").unwrap(),
    })
    .into_response()
}
async fn serve() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE items(id INTEGER PRIMARY KEY, label TEXT NOT NULL, value INTEGER NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    for id in 1_i64..=128 {
        sqlx::query("INSERT INTO items VALUES (?, ?, ?)")
            .bind(id)
            .bind(format!("row héllo {id}"))
            .bind(id.checked_mul(3).unwrap())
            .execute(&pool)
            .await
            .unwrap();
    }
    let version: String = sqlx::query_scalar("SELECT sqlite_version()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let app = Router::new()
        .route("/text", get(text))
        .route("/json", get(json))
        .route("/parse", post(parse))
        .route("/sqlite", post(sqlite))
        .layer(DefaultBodyLimit::max(65536))
        .with_state(pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    println!(
        "{}",
        serde_json::json!({"address":listener.local_addr().unwrap().to_string(),"sqliteVersion":version})
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.unwrap();
        })
        .await
        .unwrap();
    pool.close().await;
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(serve());
}
