use sea_orm::sqlx::postgres::PgPoolOptions;
use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
use std::net::TcpListener;
use std::sync::LazyLock;
use zero2prod::configuration::{DatabaseSettings, get_configuration};
use zero2prod::startup::{build, get_db_connection};
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

pub struct TestApp {
    pub address: String,
    pub db_connection: DatabaseConnection,
}

// Launch our application in the background
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    LazyLock::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let configuration = get_configuration().expect("Failed to read configuration.");

    let server = build(configuration.clone())
        .await
        .expect("Failed to build application.")
        .into_future();
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_connection: get_db_connection(&configuration.database),
    }
}

async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    let db_pool = PgPoolOptions::new().connect_lazy_with(config.with_db());
    SqlxPostgresConnector::from_sqlx_postgres_pool(db_pool)
}
