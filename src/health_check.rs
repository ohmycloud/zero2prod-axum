use axum::{http::Response, response::IntoResponse};

pub async fn health_check() -> impl IntoResponse {
    let response = Response::new("hello world");
    response.status()
}
