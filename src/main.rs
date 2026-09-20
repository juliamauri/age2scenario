use axum::{Json, Router, response::Html, routing::get};
use serde::Serialize;
use tokio::net::TcpListener;

#[derive(Serialize)]
struct HelloResponse {
    message: &'static str,
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello from Rust!",
    })
}

async fn healthz() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/hello", get(hello))
        .route("/healthz", get(healthz));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
