use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[tracing::instrument(name = "Confirm a pending subscriber")]
pub async fn confirm() -> Response {
    StatusCode::OK.into_response()
}
