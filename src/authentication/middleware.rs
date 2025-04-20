use axum::response::{IntoResponse, Redirect, Response};
use uuid::Uuid;

use crate::{session_state::TypedSession, utils::e500};

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
