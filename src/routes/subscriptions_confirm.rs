use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use entity::entities::{prelude::*, subscription_tokens, subscriptions};
use migration::SimpleExpr;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Value};
use uuid::Uuid;

use super::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, state))]
pub async fn confirm(parameters: Query<Parameters>, state: State<AppState>) -> Response {
    let id =
        match get_subscriber_id_from_token(&parameters.subscription_token, &state.db_connection)
            .await
        {
            Ok(id) => id,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

    match id {
        // Non-existing token!
        None => StatusCode::UNAUTHORIZED.into_response(),
        Some(subscriber_id) => {
            if confirm_subscriber(subscriber_id, &state.db_connection)
                .await
                .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            StatusCode::OK.into_response()
        }
    }
}

pub async fn confirm_subscriber(
    subscriber_id: Uuid,
    pool: &DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
    Subscriptions::update_many()
        .col_expr(
            subscriptions::Column::Status,
            SimpleExpr::Value(Value::String(Some(Box::new("confirmed".to_string())))),
        )
        .filter(subscriptions::Column::Id.eq(subscriber_id))
        .exec(pool)
        .await?;

    Ok(())
}

#[tracing::instrument(name = "Get subscriber ID from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    subscription_token: &str,
    pool: &DatabaseConnection,
) -> Result<Option<Uuid>, sea_orm::DbErr> {
    let token = subscription_tokens::Entity::find()
        .filter(subscription_tokens::Column::SubscriptionToken.eq(subscription_token))
        .one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    match token {
        Some(token) => Ok(Some(token.subscriber_id)),
        None => Ok(None),
    }
}
