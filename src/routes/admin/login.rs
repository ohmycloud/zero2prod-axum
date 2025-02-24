use axum::Form;
use axum::response::{Html, IntoResponse, Redirect, Response};
use secrecy::SecretString;

#[derive(Debug, serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

pub async fn login(Form(form): Form<FormData>) -> Response {
    Redirect::to("/").into_response()
}

pub async fn login_form() -> Response {
    Html(include_str!("login.html")).into_response()
}
