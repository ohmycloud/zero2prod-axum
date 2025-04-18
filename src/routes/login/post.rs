use crate::authentication::{AuthError, Credentials, validate_credentials};
use crate::routes::AppState;
use crate::routes::error_chain_fmt;
use crate::session_state::TypedSession;
use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum::response::{IntoResponse, Redirect, Response};
use axum_messages::Messages;
use hmac::{Hmac, Mac};
use secrecy::SecretString;

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Unexpected error")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        let query_string = format!("error={}", urlencoding::Encoded::new(self.to_string()));
        let secret: &[u8] = todo!();
        let hmac_tag = {
            let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret).unwrap();
            mac.update(query_string.as_bytes());
            mac.finalize().into_bytes()
        };

        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(LOCATION, format!("/login?{query_string}&tag={hmac_tag:x}"))
            .body(axum::body::Body::empty())
            .unwrap()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    skip(form, state, flash, session),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<AppState>,
    flash: Messages,
    session: TypedSession,
    Form(form): Form<FormData>,
) -> Result<Response, Response> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &state.db_connection).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            let redirect_err = move |e: tower_sessions::session::Error| {
                login_redirect(flash.clone(), LoginError::UnexpectedError(e.into()))
            };
            session
                .insert_iser_id(user_id)
                .await
                .map_err(&redirect_err)?;
            Ok(Redirect::to("/admin/dashboard").into_response())
        }
        Err(error) => {
            let error = match error {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(error.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(error.into()),
            };

            flash.error(error.to_string());

            Err(Redirect::to("/login").into_response())
        }
    }
}

// Redirect to the login page with an error message.
fn login_redirect(flash: Messages, err: LoginError) -> Response {
    flash.error(err.to_string());
    Redirect::to("/login").into_response()
}
