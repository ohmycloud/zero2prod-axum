use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

pub async fn admin_dashboard() -> Response {
    StatusCode::OK.into_response()
}
