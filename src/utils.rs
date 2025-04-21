use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

pub fn e500<T>(e: T) -> Response
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    tracing::error!("{:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

// Return a 400 with the user-representation of the validation error as body.
// The error root cause is preserved for logging purposes.
pub fn e400<T>(e: T) -> Response
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    tracing::error!("{:?}", e);
    StatusCode::BAD_REQUEST.into_response()
}
