use axum::{
    Form,
    response::{IntoResponse, Redirect, Response},
};
use axum_messages::Messages;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, SecretString};

use crate::{session_state::TypedSession, utils::e500};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

pub async fn change_password(
    flash: Messages,
    session: TypedSession,
    Form(form): Form<FormData>,
) -> Result<Response, StatusCode> {
    if session.get_user_id().await.map_err(e500)?.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    // SecretString does not implement `Eq`,
    // therefore we need to compare the underlying `String`
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        let _flash = flash.error(
            "You entered two different new passwords - \
             the field values must match.",
        );
        return Ok(Redirect::to("/admin/password").into_response());
    }
    return Ok(Redirect::to("/login").into_response());
}
