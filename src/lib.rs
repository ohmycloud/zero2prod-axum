pub mod health_check;

use crate::health_check::health_check;
use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{IntoMakeService, get, post},
    serve::Serve,
};
use tokio::net::TcpListener;

async fn index() -> impl IntoResponse {
    "Rust Rocks!"
}

async fn greet(Path(name): Path<String>) -> impl IntoResponse {
    format!("Hello, {}", name)
}

async fn subscribe() -> Response {
    StatusCode::OK.into_response()
}

pub fn run(
    listener: std::net::TcpListener,
) -> Result<Serve<TcpListener, IntoMakeService<Router>, Router>, std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/", get(index))
        .route("/{name}", get(greet));

    let listener = TcpListener::from_std(listener)?;
    println!("Listening on {:?}", listener.local_addr());

    let server = axum::serve(listener, app.into_make_service());
    Ok(server)
}
