use crate::helpers::assert_is_redirect_to;
use crate::helpers::spawn_app;
use axum::http::HeaderValue;
use std::collections::HashSet;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let login_body = serde_json::json!({
       "username": "random-username",
       "password": "random-password",
    });

    let response = app.post_login(&login_body).await;

    let cookies: HashSet<_> = response
        .headers()
        .get_all("Set-Cookie")
        .into_iter()
        .collect();

    // Assert
    assert!(cookies.contains(&HeaderValue::from_str("_flash=Authentication failed").unwrap()));
    let flash_cookie = response.cookies().find(|c| c.name() == "_flash").unwrap();
    assert_eq!(flash_cookie.value(), "Authentication failed");
    assert_is_redirect_to(&response, "/login");
}
