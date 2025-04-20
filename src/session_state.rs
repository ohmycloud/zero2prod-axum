use axum::{extract::FromRequestParts, http::request::Parts};
use reqwest::StatusCode;
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub async fn insert_iser_id(
        &self,
        user_id: Uuid,
    ) -> Result<(), tower_sessions::session::Error> {
        self.0.insert(Self::USER_ID_KEY, user_id).await
    }

    pub async fn get_user_id(&self) -> Result<Option<Uuid>, tower_sessions::session::Error> {
        self.0.get(Self::USER_ID_KEY).await
    }

    pub async fn cycle_id(&self) -> Result<(), tower_sessions::session::Error> {
        self.0.cycle_id().await
    }

    pub async fn log_out(self) -> Result<(), tower_sessions::session::Error> {
        self.0.flush().await
    }
}

impl<S> FromRequestParts<S> for TypedSession
where
    S: Send + Sync,
{
    #[doc = " If the extractor fails it\'ll use this \"rejection\" type. A rejection is"]
    #[doc = " a kind of error that can be converted into a response."]
    type Rejection = (StatusCode, &'static str);

    #[doc = " Perform the extraction."]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        Ok(Self(session))
    }
}
