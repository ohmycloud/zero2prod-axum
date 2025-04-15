use crate::authentication::{AuthError, Credentials, validate_credentials};
use crate::routes::{AppState, error_chain_fmt};
use crate::startup::HmacSecret;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Form, http};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use handlebars::Handlebars;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};

#[derive(Debug, serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

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
            .header(
                http::header::LOCATION,
                format!("/login?{query_string}&tag={hmac_tag:x}"),
            )
            .body(axum::body::Body::empty())
            .unwrap()
    }
}

#[tracing::instrument(
    skip(form, state),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<AppState>,
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
            Ok(Redirect::to("/").into_response())
        }
        Err(error) => {
            let error = match error {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(error.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(error.into()),
            };

            let cookie = Cookie::new("_flash", error.to_string());
            let mut response = Redirect::to("/login").into_response();

            response.headers_mut().insert(
                axum::http::header::SET_COOKIE,
                cookie.to_string().parse().unwrap(),
            );
            Err(response)
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));
        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}

pub async fn login_form(jar: CookieJar) -> (StatusCode, CookieJar, Html<String>) {
    let error_html = match jar.get("_flash") {
        Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()),
        None => "".into(),
    };
    let html_template = include_str!("login.html");
    let reg = Handlebars::new();
    let login_form = reg
        .render_template(html_template, &error_html)
        .expect("Failed to render login form.");

    let jar = jar.remove("_flash");
    (StatusCode::OK, jar, Html::from(login_form))
}
