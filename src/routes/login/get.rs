use axum::http::StatusCode;
use axum::response::Html;
use axum_messages::Messages;
use handlebars::Handlebars;
use std::fmt::Write;

pub async fn login_form(flash: Messages) -> (StatusCode, Html<String>) {
    let mut error_html = String::new();
    for message in flash.into_iter() {
        writeln!(error_html, "<p><i>{}</i></p>", message.message).unwrap();
    }
    let html_template = include_str!("login.html");
    let reg = Handlebars::new();
    let login_form = reg
        .render_template(html_template, &error_html)
        .expect("Failed to render login form.");

    (StatusCode::OK, Html::from(login_form))
}
