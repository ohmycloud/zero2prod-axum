// `tokio::test` is the testing equivalent of `tokio::main`.
// Is also spares you from having to specify the `#[test]` attribute.
//
// You can inspect what code gets generated using
// `cargo expand --test health_check`
#[tokio::test]
async fn health_check_works() {
    // Arrange
    spawn_app();

    // We need to bring in `reqwest`
    // to perform HTTP requsts against our application.
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:3333/health_check")
        .send()
        .await
        .expect("Falied to execute reqwest.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Launch our application in the background
fn spawn_app() {
    let server = zero2prod::run("127.0.0.1:0")
        .expect("Failed to bind address")
        .into_future();
    let _ = tokio::spawn(server);
    println!("app spawned.");
}
