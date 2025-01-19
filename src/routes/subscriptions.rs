use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use entity::entities::prelude::*;
use entity::entities::subscriptions;
use sea_orm::{DatabaseConnection, EntityTrait, Set, prelude::DateTimeWithTimeZone};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db_connection: DatabaseConnection,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, form),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
pub async fn subscribe(State(state): State<AppState>, Form(form): Form<FormData>) -> Response {
    match insert_subscriber(&state, &form).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(state, form)
)]
pub async fn insert_subscriber(state: &AppState, form: &FormData) -> Result<(), sea_orm::DbErr> {
    let subscriber_id = Uuid::new_v4();
    let subscription = subscriptions::ActiveModel {
        id: Set(subscriber_id),
        name: Set(form.name.clone()),
        email: Set(form.email.clone()),
        subscribed_at: Set(DateTimeWithTimeZone::from(Utc::now())),
        status: Set("confirmed".to_string()),
    };

    Subscriptions::insert(subscription)
        .exec(&state.db_connection)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}
