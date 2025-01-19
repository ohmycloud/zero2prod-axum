use crate::routes::{AppState, health_check::*, subscribe};
use axum::{
    Router,
    body::Body,
    http::Request,
    routing::{IntoMakeService, get, post},
    serve::Serve,
};
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tower_request_id::{RequestId, RequestIdLayer};
use tracing::info_span;

pub fn run(
    listener: std::net::TcpListener,
    connection: DatabaseConnection,
) -> Result<Serve<TcpListener, IntoMakeService<Router>, Router>, std::io::Error> {
    let app_state = AppState {
        db_connection: connection,
    };
    let app = Router::new()
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                // We get the request id from the extensions
                let request_id = request
                    .extensions()
                    .get::<RequestId>()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "unknown".into());
                // And then we put it along with other information into the `request` span
                info_span!(
                    "request",
                    id = %request_id,
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
        .layer(RequestIdLayer)
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/", get(index))
        .route("/{name}", get(greet))
        .with_state(app_state.clone());

    let listener = TcpListener::from_std(listener)?;
    println!(
        "Listening on http://{:?}",
        listener.local_addr().expect("network error")
    );

    let server = axum::serve(listener, app.into_make_service());
    Ok(server)
}
