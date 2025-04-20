use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_messages::Messages;
use handlebars::Handlebars;
use reqwest::StatusCode;
use std::fmt::Write;

use crate::{session_state::TypedSession, utils::e500};

pub async fn change_password_form(
    flash: Messages,
    session: TypedSession,
) -> Result<Response, StatusCode> {
    if session.get_user_id().await.map_err(e500)?.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    let mut error_html = String::new();
    for m in flash.into_iter() {
        writeln!(error_html, "<p><i>{}</i></p>", m.message).unwrap();
    }

    let reg = Handlebars::new();
    let html = reg
        .render_template(
            include_str!("get.html"),
            &serde_json::json!({
                "error_html": error_html
            }),
        )
        .expect("Failed to render pasword page.");

    Ok(Html::from(html).into_response())
}
