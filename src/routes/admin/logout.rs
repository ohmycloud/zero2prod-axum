use axum::{
    Extension,
    response::{IntoResponse, Redirect, Response},
};
use axum_messages::Messages;

use crate::{authentication::UserId, session_state::TypedSession};

pub async fn log_out(
    flash: Messages,
    session: TypedSession,
    user_id: Extension<UserId>,
) -> Result<Response, Response> {
    let _user_id = *(user_id.0);
    session.log_out().await.expect("Failed to logout");
    flash.info("You have successfully logged out.");
    Ok(Redirect::to("/login").into_response())
}
