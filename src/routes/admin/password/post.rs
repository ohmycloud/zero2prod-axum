use axum::{
    Extension, Form,
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_messages::Messages;
use secrecy::{ExposeSecret, SecretString};

use crate::{
    authentication::{self, AuthError, Credentials, UserId, validate_credentials},
    routes::{AppState, get_username},
    utils::e500,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

pub async fn change_password(
    State(state): State<AppState>,
    flash: Messages,
    user_id: Extension<UserId>,
    Form(form): Form<FormData>,
) -> Result<Response, Response> {
    let user_id = user_id.0;
    // SecretString does not implement `Eq`,
    // therefore we need to compare the underlying `String`
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        flash.error(
            "You entered two different new passwords - \
             the field values must match.",
        );
        return Ok(Redirect::to("/admin/password").into_response());
    }
    if form.new_password.expose_secret().len() < 12 {
        flash.error("You password is too short!");
        return Ok(Redirect::to("/admin/password").into_response());
    }
    let username = get_username(*user_id, &state.db_connection)
        .await
        .map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.current_password,
    };

    if let Err(e) = validate_credentials(credentials, &state.db_connection).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                flash.error("The current password is incorrect.");
                Ok(Redirect::to("/admin/password").into_response())
            }
            AuthError::UnexpectedError(_) => Err(e500(e.to_string())),
        };
    }

    authentication::change_password(*user_id, form.new_password, &state.db_connection)
        .await
        .map_err(e500)?;

    flash.success("Your password has been changed.");

    return Ok(Redirect::to("/admin/password").into_response());
}
