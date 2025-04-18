use anyhow::Context;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};
use entity::entities::prelude::Users;
use handlebars::Handlebars;
use reqwest::StatusCode;
use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;

use crate::{routes::AppState, session_state::TypedSession};

pub async fn admin_dashboard(
    State(state): State<AppState>,
    session: TypedSession,
) -> Result<Response, StatusCode> {
    let username = if let Some(user_id) = session.get_user_id().await.map_err(e500)? {
        get_username(user_id, &state.db_connection)
            .await
            .map_err(e500)?
    } else {
        return Ok(Redirect::to("/login").into_response());
    };
    let reg = Handlebars::new();
    let html = reg
        .render_template(
            include_str!("dashboard.html"),
            &serde_json::json!({"username": username}),
        )
        .map_err(e500)?;
    Ok((StatusCode::OK, Html::from(html)).into_response())
}

fn e500<T>(e: T) -> StatusCode
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    tracing::error!("{:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

#[tracing::instrument(name = "Get username", skip(conn))]
async fn get_username(user_id: Uuid, conn: &DatabaseConnection) -> Result<String, anyhow::Error> {
    let user = Users::find_by_id(user_id)
        .one(conn)
        .await
        .context("Failed to perform a query to retrieve a username.")?;
    user.map_or(Err(anyhow::anyhow!("User not found")), |user| {
        Ok(user.username)
    })
}
