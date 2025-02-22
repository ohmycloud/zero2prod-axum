use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use entity::entities::{prelude::*, subscriptions};
use reqwest::StatusCode;
use sea_orm::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};

use super::{AppState, error_chain_fmt};

#[derive(Debug, serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Debug, serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

pub async fn publish_newsletter(
    State(state): State<AppState>,
    Json(body): Json<BodyData>,
) -> Result<Response, PublishError> {
    let _subscribers = get_confirmed_subscribers(&state.db_connection).await?;
    Ok(StatusCode::OK.into_response())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(db_connection))]
async fn get_confirmed_subscribers(
    db_connection: &DatabaseConnection,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let subscribers = Subscriptions::find()
        .filter(subscriptions::Column::Status.eq("confirmed"))
        .all(db_connection)
        .await
        .expect("Failed to fetch data.")
        .into_iter()
        .map(|model| ConfirmedSubscriber { email: model.email })
        .collect::<Vec<ConfirmedSubscriber>>();

    Ok(subscribers)
}
