pub mod health_check;

use crate::health_check::health_check;
use axum::{Router, extract::Path, response::IntoResponse, routing::get};
use tokio::net::TcpListener;

async fn index() -> impl IntoResponse {
    "Rust Rocks!"
}

async fn greet(Path(name): Path<String>) -> impl IntoResponse {
    format!("Hello, {}", name)
}

pub async fn run() -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/", get(index))
        .route("/{name}", get(greet));

    let listener = TcpListener::bind("0.0.0.0:3333").await.unwrap();
    println!("Listening on {:?}", listener.local_addr());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
