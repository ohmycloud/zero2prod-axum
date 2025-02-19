use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
pub async fn confirm(parameters: Query<Parameters>) -> Response {
    StatusCode::OK.into_response()
}
