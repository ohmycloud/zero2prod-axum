use axum::response::{IntoResponse, Redirect, Response};
use axum_messages::Messages;

use crate::{session_state::TypedSession, utils::reject_anonymous_users};

pub async fn log_out(flash: Messages, session: TypedSession) -> Result<Response, Response> {
    let _user_id = reject_anonymous_users(session.clone()).await?;
    session.log_out().await.expect("Failed to logout");
    flash.info("You have successfully logged out.");
    Ok(Redirect::to("/login").into_response())
}
