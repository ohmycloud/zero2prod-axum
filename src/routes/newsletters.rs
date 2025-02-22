use axum::response::{IntoResponse, Response};
use entity::entities::subscriptions::Entity;
use entity::entities::{prelude::*, subscriptions};
use reqwest::StatusCode;
use sea_orm::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};

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

// Dummy implementation
pub async fn publish_newsletter(_body: axum::Json<BodyData>) -> Response {
    StatusCode::OK.into_response()
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
