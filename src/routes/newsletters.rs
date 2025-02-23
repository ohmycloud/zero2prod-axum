use super::{AppState, error_chain_fmt};
use crate::domain::SubscriberEmail;
use anyhow::Context;
use axum::extract::{Json, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use entity::entities::{prelude::*, subscriptions, users};
use reqwest::StatusCode;
use sea_orm::prelude::*;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use secrecy::{ExposeSecret, SecretString};
use sha3::Digest;

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

struct Credentials {
    username: String,
    password: SecretString,
}

async fn validate_credentials(
    credentials: Credentials,
    db_connection: &DatabaseConnection,
) -> Result<uuid::Uuid, PublishError> {
    let password_hash = sha3::Sha3_256::digest(credentials.password.expose_secret().as_bytes());
    // Lowercase hexadecimal encoding.
    let password_hash = format!("{:x}", password_hash);
    let user_id = Users::find()
        .filter(users::Column::Username.eq(credentials.username))
        .filter(users::Column::PasswordHash.eq(password_hash))
        .one(db_connection)
        .await
        .context("Failed to perform a query to validate auth credentials.")
        .map_err(PublishError::UnexpectedError)?;

    user_id
        .map(|r| r.user_id)
        .ok_or_else(|| anyhow::anyhow!("Invalid username or password."))
        .map_err(PublishError::AuthError)
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    // The header value, if present, must be a valid UTF8 string
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization schema was not 'Basic'.")?;
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-deocde 'Basic' credentials.")?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The deocded credential string is not valid UTF8.")?;

    // Split into two segments, using ':' as delimiter
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: SecretString::new(Box::from(password)),
    })
}

#[tracing::instrument(
    name = "Publishing a newsletter issue",
    skip(headers, state, body),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<BodyData>,
) -> Result<Response, PublishError> {
    // Bubble up the error, performing the necessary conversion
    let credentials = basic_authentication(&headers).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &state.db_connection).await?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    let subscribers = get_confirmed_subscribers(&state.db_connection).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                state
                    .email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain= ?error,
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
