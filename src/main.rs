use axum::{Router, extract::Path, response::IntoResponse, routing::get};
use tokio::net::TcpListener;

async fn greet(Path(name): Path<String>) -> impl IntoResponse {
    format!("Hello, {}", name)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/{name}", get(greet));
    let listener = TcpListener::bind("0.0.0.0:3333").await.unwrap();
    println!("Listening on {:?}", listener.local_addr());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
