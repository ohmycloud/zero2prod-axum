use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use entity::entities::subscription_tokens;
use entity::entities::subscriptions;
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set, TransactionTrait,
    prelude::DateTimeWithTimeZone,
};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[derive(Serialize, Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db_connection: DatabaseConnection,
    pub email_client: EmailClient,
    pub base_url: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(Self { email, name })
    }
}

pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(form.name)?;
    let email = SubscriberEmail::parse(form.email)?;
    Ok(NewSubscriber { email, name })
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
    let new_subscriber = match form.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let mut transaction = match state.db_connection.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let subscription_token = generate_subscription_token();
    if store_token(&mut transaction, &subscription_token, subscriber_id)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // Send a (useless) email to the new subscriber.
    // We are ignoring email delivery errors for now.
    if send_confirmation_email(
        state.email_client,
        new_subscriber,
        state.base_url,
        subscription_token.as_str(),
    )
    .await
    .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::OK.into_response()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(transaction, new_subscriber)
)]
pub async fn insert_subscriber(
    transaction: &DatabaseTransaction,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sea_orm::DbErr> {
    let subscriber_id = Uuid::new_v4();
    let subscription = subscriptions::ActiveModel {
        id: Set(subscriber_id),
        name: Set(new_subscriber.name.as_ref().to_string()),
        email: Set(new_subscriber.email.as_ref().to_string()),
        subscribed_at: Set(DateTimeWithTimeZone::from(Utc::now())),
        status: Set("pending_confirmation".to_string()),
    };

    subscription.insert(transaction).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: EmailClient,
    new_subscriber: NewSubscriber,
    base_url: String,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let palin_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &palin_body)
        .await
}

#[tracing::instrument(
    name = "Storing subscription token in the database",
    skip(transaction, subscriber_id, subscription_token)
)]
pub async fn store_token(
    transaction: &DatabaseTransaction,
    subscription_token: &str,
    subscriber_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    let token = subscription_tokens::ActiveModel {
        subscription_token: Set(subscription_token.to_string()),
        subscriber_id: Set(subscriber_id),
    };

    token.insert(transaction).await.map_err(|e| {
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

/// Generate a random 25-characters-long case-sensitive subscription token.
pub fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
