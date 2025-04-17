use crate::startup::HmacSecret;
use axum::http::StatusCode;
use axum::response::Html;
use axum_extra::extract::CookieJar;
use handlebars::Handlebars;
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;

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
