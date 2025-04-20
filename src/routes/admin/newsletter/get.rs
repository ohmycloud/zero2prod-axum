use axum::{
    Extension,
    response::{Html, IntoResponse, Response},
};
use axum_messages::Messages;
use handlebars::Handlebars;
use std::fmt::Write;

use crate::authentication::UserId;

pub async fn publish_newsletter_form(
    flash: Messages,
    user_id: Extension<UserId>,
) -> Result<Response, Response> {
    let _user_id = *(user_id.0);
    let mut msg_html = String::new();
    for m in flash.into_iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.message).unwrap();
    }
    let reg = Handlebars::new();
    let html = reg
        .render_template(
            include_str!("get.html"),
            &serde_json::json!({
                "messages": msg_html,
            }),
        )
        .expect("Failed to render password page.");

    Ok(Html::from(html).into_response())
}
