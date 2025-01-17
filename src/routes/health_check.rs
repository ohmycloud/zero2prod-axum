use axum::{
    extract::Path,
    response::{IntoResponse, Response},
};

pub async fn health_check() -> impl IntoResponse {
    let response = Response::new("hello world");
    response.status()
}

pub async fn index() -> impl IntoResponse {
    "Rust Rocks!"
}

pub async fn greet(Path(name): Path<String>) -> impl IntoResponse {
    format!("Hello, {}", name)
}
