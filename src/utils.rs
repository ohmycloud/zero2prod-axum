use axum::response::{IntoResponse, Redirect, Response};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::session_state::TypedSession;

pub fn e500<T>(e: T) -> Response
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    tracing::error!("{:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

pub async fn reject_anonymous_users(session: TypedSession) -> Result<Uuid, Response> {
    match session.get_user_id().await.map_err(e500)? {
        Some(user_id) => Ok(user_id),
        None => {
            let e = anyhow::anyhow!("The user has not logged in");
            tracing::error!(error = %e, "The user has not logged in");
            Err(Redirect::to("/login").into_response())
        }
    }
}
