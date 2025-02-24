use axum::response::{Html, IntoResponse, Redirect, Response};

pub async fn login() -> Response {
    Redirect::to("/").into_response()
}

pub async fn login_form() -> Response {
    Html(include_str!("login.html")).into_response()
}
