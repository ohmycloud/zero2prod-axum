use axum::{
    Form, debug_handler,
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

#[debug_handler]
pub async fn subscribe(State(state): State<AppState>, Form(form): Form<FormData>) -> Response {
    let subscriber_id = Uuid::new_v4();
    let subscription = subscriptions::ActiveModel {
        id: Set(subscriber_id),
        name: Set(form.name.clone()),
        email: Set(form.email.clone()),
        subscribed_at: Set(DateTimeWithTimeZone::from(Utc::now())),
        status: Set("confirmed".to_string()),
    };

    let request_id = Uuid::new_v4();

    log::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber.",
        request_id,
        form.name,
        form.email
    );
    log::info!(
        "request_id {} - Saving new subscriber details in the database",
        request_id
    );
    let query = Subscriptions::insert(subscription)
        .exec(&state.db_connection)
        .await;
    match query {
        Ok(_) => {
            log::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            StatusCode::OK.into_response()
        }
        Err(e) => {
            log::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
