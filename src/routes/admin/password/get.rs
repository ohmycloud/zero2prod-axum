use axum::{
    Extension,
    response::{Html, IntoResponse, Response},
};
use axum_messages::Messages;
use handlebars::Handlebars;
use std::fmt::Write;

use crate::authentication::UserId;

pub async fn change_password_form(
    flash: Messages,
    user_id: Extension<UserId>,
) -> Result<Response, Response> {
    let _user_id = *(user_id.0);

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
