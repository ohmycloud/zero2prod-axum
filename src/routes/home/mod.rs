use axum::response::{Html, IntoResponse, Response};

pub async fn home() -> Response {
    Html(include_str!("home.html")).into_response()
}
