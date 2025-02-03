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
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

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
    let name = match SubscriberName::parse(form.name) {
        Ok(name) => name,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let email = match SubscriberEmail::parse(form.email) {
        Ok(email) => email,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let new_subscriber = NewSubscriber { email, name };

    match insert_subscriber(&state, &new_subscriber).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(state, new_subscriber)
)]
pub async fn insert_subscriber(
    state: &AppState,
    new_subscriber: &NewSubscriber,
) -> Result<(), sea_orm::DbErr> {
    let subscriber_id = Uuid::new_v4();
    let subscription = subscriptions::ActiveModel {
        id: Set(subscriber_id),
        name: Set(new_subscriber.name.as_ref().to_string()),
        email: Set(new_subscriber.email.as_ref().to_string()),
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

/// Returns `true` if the input satisfies all our validation constraints
/// on subscriber names, `false` otherwise.
pub fn is_valid_name(s: &str) -> bool {
    // `.trim()` returns a view over the input `s` without trailing
    // whitespace-like characters.
    // `.is_empty` checks if the view contains any character.
    let is_empty_or_whitespace = s.trim().is_empty();
    // A grapheme is defined by the Unicode standard as a "user-perceived"
    // character: `å` is a single grapheme, but it is composed of two characters
    // (`a` and `̊`).
    //
    // `graphemes` returns an iterator over the graphemes in the input `s`.
    // `true` specifies that we want to use the extended grapheme definition set,
    // the recommended one.
    let is_too_long = s.graphemes(true).count() > 256;
    // Iterate over all characters in the input `s` to check if any of them matches
    // one of the characters in the forbidden array.
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    // Return `false` if any of our conditions have been violated
    !(is_empty_or_whitespace | is_too_long || contains_forbidden_characters)
}
