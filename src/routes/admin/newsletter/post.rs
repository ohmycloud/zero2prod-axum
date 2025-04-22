use crate::authentication::UserId;
use crate::domain::SubscriberEmail;
use crate::idempotency::{IdempotencyKey, get_saved_response};
use crate::routes::{AppState, error_chain_fmt};
use crate::utils::{e400, e500};
use anyhow::Context;
use axum::extract::State;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Form};
use axum_messages::Messages;
use entity::entities::{prelude::*, subscriptions};
use reqwest::StatusCode;
use sea_orm::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};

#[derive(Debug, serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
    idempotency_key: String,
}

#[derive(Debug)]
struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
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
            PublishError::AuthError(_) => {
                let mut response = StatusCode::UNAUTHORIZED.into_response();
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();

                response
                    .headers_mut()
                    .insert("WWW-Authenticate", header_value);
                response
            }
        }
    }
}

#[tracing::instrument(
    name = "Publishing a newsletter issue",
    skip(state, user_id, form),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    flash: Messages,
    user_id: Extension<UserId>,
    Form(form): Form<FormData>,
) -> Result<Response, PublishError> {
    let idempotency_key: IdempotencyKey = form.idempotency_key.try_into().map_err(e400).unwrap();
    // Return early if er have a saved response in the database
    if let Some(saved_response) =
        get_saved_response(&state.db_connection, &idempotency_key, *user_id.0)
            .await
            .map_err(e500)
            .unwrap()
    {
        flash.info("The newsletter issue has been published!");
        return Ok(saved_response);
    }

    tracing::info!("Publishing a newsletter issue: {}", *user_id);
    let subscribers = get_confirmed_subscribers(&state.db_connection).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                state
                    .email_client
                    .send_email(
                        &subscriber.email,
                        &form.title,
                        &form.html_content,
                        &form.text_content,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
                )
            }
        }
    }
    Ok(StatusCode::OK.into_response())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(db_connection))]
async fn get_confirmed_subscribers(
    db_connection: &DatabaseConnection,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let subscribers = Subscriptions::find()
        .filter(subscriptions::Column::Status.eq("confirmed"))
        .all(db_connection)
        .await
        .expect("Failed to fetch data.")
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect::<Vec<Result<ConfirmedSubscriber, anyhow::Error>>>();

    Ok(subscribers)
}
