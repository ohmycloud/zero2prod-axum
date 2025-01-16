use axum::{
    Form, Router,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{IntoMakeService, get, post},
    serve::Serve,
};
use serde::Deserialize;
use tokio::net::TcpListener;

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

pub async fn subscribe() -> Response {
    StatusCode::OK.into_response()
}

#[derive(Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub async fn home(Form(form): Form<FormData>) -> Response {
    format!("Welcome: {}", form.name).into_response()
}
