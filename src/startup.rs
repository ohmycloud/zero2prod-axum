use crate::routes::{health_check::*, subscribe};
use axum::{
    Router,
    routing::{IntoMakeService, get, post},
    serve::Serve,
};
use tokio::net::TcpListener;

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
