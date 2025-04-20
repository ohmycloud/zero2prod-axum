use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

pub fn e500<T>(e: T) -> Response
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    tracing::error!("{:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}
