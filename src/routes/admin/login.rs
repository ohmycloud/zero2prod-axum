use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

pub async fn login_form() -> Response {
    StatusCode::OK.into_response()
}
