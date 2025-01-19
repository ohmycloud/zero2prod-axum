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
use tracing::Instrument;
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

    // spans, like logs, have an associated level
    // `info_span` creates a span at the info-level
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        request_id = %request_id,
        subscriber_name = %form.name,
        subscriber_email = %form.email
    );
    // using `enter` in an async function is a recipe for disaster!
    // bear with me for now, but don't do this at home.
    let _request_span_guard = request_span.enter();

    // We do not call `.enter` on query_span!
    // `.instrument` takes care of it at the right moments
    // in the query future lifetime
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    let query = Subscriptions::insert(subscription)
        .exec(&state.db_connection)
        // First we attach the instrumentation, then we `.await` it
        .instrument(query_span)
        .await;
    match query {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            // Yes, this error log falls outside of `query_span`
            // We'll rectify it later, pinky swear!
            tracing::error!("Failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
