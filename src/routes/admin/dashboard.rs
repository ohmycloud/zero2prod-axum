use anyhow::Context;
use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse, Response},
};
use entity::entities::prelude::Users;
use handlebars::Handlebars;
use reqwest::StatusCode;
use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;

use crate::{authentication::UserId, routes::AppState, utils::e500};

pub async fn admin_dashboard(
    State(state): State<AppState>,
    user_id: Extension<UserId>,
) -> Result<Response, Response> {
    let user_id = user_id.0;
    let username = get_username(*user_id, &state.db_connection)
        .await
        .map_err(e500)?;

    let reg = Handlebars::new();
    let html = reg
        .render_template(
            include_str!("dashboard.html"),
            &serde_json::json!({"username": username}),
        )
        .map_err(e500)?;
    Ok((StatusCode::OK, Html::from(html)).into_response())
}

#[tracing::instrument(name = "Get username", skip(conn))]
pub async fn get_username(
    user_id: Uuid,
    conn: &DatabaseConnection,
) -> Result<String, anyhow::Error> {
    let user = Users::find_by_id(user_id)
        .one(conn)
        .await
        .context("Failed to perform a query to retrieve a username.")?;
    user.map_or(Err(anyhow::anyhow!("User not found")), |user| {
        Ok(user.username)
    })
}
