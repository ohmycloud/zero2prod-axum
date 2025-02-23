use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher};
use entity::entities::users;
use sea_orm::sqlx::postgres::PgPoolOptions;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set, SqlxPostgresConnector};
use std::net::TcpListener;
use std::sync::LazyLock;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::startup::Application;
use zero2prod::startup::get_db_connection;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber_name = "test".to_string();
    let default_filter_level = "info".to_string();

    // We cannot assign the output of `get_subscriber` to a variable based on the
    // value `TEST_LOG` because the sink is part of the type returned by
    // `get_subscriber`, therefore they are not the same type. We could work around
    // it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_connection: DatabaseConnection,
    pub email_server: MockServer,
    pub test_user: TestUser,
}

/// Confirmation links embedded in the request to the email API
#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    async fn store(&self, db_connection: &DatabaseConnection) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        // Match parameters of the default password
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.test_user.password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    }
    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Extract the confirmation links embedded in the request to the email API.
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }
}

#[derive(Debug)]
pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(&self, db_connection: &DatabaseConnection) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        // We don't care about the exact Argon2 parameters here
        // given that it's for testing purposes
        let password_hash = Argon2::default()
            .hash_password(self.password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string();

        let user = users::ActiveModel {
            user_id: Set(self.user_id),
            username: Set(self.username.clone()),
            password_hash: Set(password_hash),
        };
        user.insert(db_connection)
            .await
            .expect("Failed to store test user.");
    }
}

// Launch our application in the background
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    LazyLock::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    listener.set_nonblocking(true).unwrap();

    // Launch a mock server to stand in for Postmark's API
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a different database for each test case
        // c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        // Use the mock server as email API
        c.email_client.base_url = email_server.uri();
        c
    };

    // Create and migrate the database
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();

    // Got the port before spawning the application
    let address = format!("http://127.0.0.1:{}", application_port);

    let _ = tokio::spawn(application.run_until_stopped());

    let test_app = TestApp {
        address,
        port: application_port,
        db_connection: get_db_connection(&configuration.database),
        email_server,
        test_user: TestUser::generate(),
    };

    test_app.test_user.store(&test_app.db_connection).await;

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    let db_pool = PgPoolOptions::new().connect_lazy_with(config.with_db());
    SqlxPostgresConnector::from_sqlx_postgres_pool(db_pool)
}
