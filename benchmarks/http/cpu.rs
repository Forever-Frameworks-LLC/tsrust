// Independent Rust rendering kernel. Same f64 operations; no precomputed pixels.
use axum::{Router, body::Bytes, extract::DefaultBodyLimit, http::StatusCode, response::{IntoResponse, Response}, routing::post};
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
struct Input { id: i64 }
#[derive(Serialize)]
struct Output { checksum: i64, escaped: i64, id: i64 }
fn render(id: i64) -> Output {
    let mut checksum = 0_i64;
    let mut escaped = 0_i64;
    for y in 0_i64..64 {
        for x in 0_i64..96 {
            let cx = -2.0 + x as f64 * 3.0 / 96.0 + (id % 7) as f64 * 0.0001;
            let cy = -1.0 + y as f64 * 2.0 / 64.0 + (id % 11) as f64 * 0.0001;
            let (mut zx, mut zy, mut iterations) = (0.0, 0.0, 0_i64);
            while iterations < 256 && zx * zx + zy * zy <= 4.0 {
                let next = zx * zx - zy * zy + cx;
                zy = 2.0 * zx * zy + cy; zx = next; iterations += 1;
            }
            checksum = (checksum + iterations * (y * 96 + x + 1)) % 1_000_000_007;
            if iterations < 256 { escaped += 1; }
        }
    }
    Output { id, checksum, escaped }
}
async fn handle(body: Bytes) -> Response {
    let Ok(input) = serde_json::from_slice::<Input>(&body) else { return (StatusCode::BAD_REQUEST, "invalid").into_response() };
    if !(1..=128).contains(&input.id) { return (StatusCode::BAD_REQUEST, "invalid").into_response() }
    ([("content-type", "application/json")], serde_json::to_vec(&render(input.id)).unwrap()).into_response()
}
fn main() {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
        let app = Router::new().route("/render", post(handle)).layer(DefaultBodyLimit::max(65536));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        println!("{}", serde_json::json!({"address": listener.local_addr().unwrap().to_string()}));
        axum::serve(listener, app).with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() }).await.unwrap();
    });
}
