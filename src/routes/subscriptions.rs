use axum::{Form, http::StatusCode, response::IntoResponse, response::Response};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub async fn subscribe(Form(form): Form<FormData>) -> Response {
    StatusCode::OK.into_response()
}
