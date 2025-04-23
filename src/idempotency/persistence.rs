use axum::response::{IntoResponse, Response};
use entity::entities::{idempotency, prelude::Idempotency};
use reqwest::StatusCode;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use super::IdempotencyKey;

pub async fn get_saved_response(
    db_connection: &DatabaseConnection,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<Response>, anyhow::Error> {
    let saved_response = Idempotency::find()
        .filter(idempotency::Column::UserId.eq(user_id))
        .filter(idempotency::Column::IdempotencyKey.eq(idempotency_key.as_ref()))
        .one(db_connection)
        .await?;
    if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;
        let mut response = Response::new(status_code);
    }
    Ok(Some("".into_response()))
}

pub async fn save_response(
    db_connection: &DatabaseConnection,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    http_response: &Response,
) -> Result<(), anyhow::Error> {
    Ok(())
}
