use axum::{
    Form,
    response::{IntoResponse, Redirect, Response},
};
use reqwest::StatusCode;
use secrecy::SecretString;

use crate::{session_state::TypedSession, utils::e500};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

pub async fn change_password(
    session: TypedSession,
    Form(form): Form<FormData>,
) -> Result<Response, StatusCode> {
    if session.get_user_id().await.map_err(e500)?.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    todo!()
}
