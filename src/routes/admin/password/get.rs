use axum::response::{Html, IntoResponse, Response};
use axum_messages::Messages;
use handlebars::Handlebars;
use std::fmt::Write;

use crate::{authentication::reject_anonymous_users, session_state::TypedSession};

pub async fn change_password_form(
    flash: Messages,
    session: TypedSession,
) -> Result<Response, Response> {
    let _user_id = reject_anonymous_users(session).await?;

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
