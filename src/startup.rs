use crate::{
    configuration::{DatabaseSettings, Settings, get_configuration},
    email_client::EmailClient,
    routes::{
        AppState, admin_dashboard, change_password, change_password_form, confirm, greet,
        health_check, home, index, login, login_form, publish_newsletter, subscribe,
    },
};
use axum::{
    Router,
    routing::{IntoMakeService, get, post},
    serve::Serve,
};
use axum_messages::MessagesManagerLayer;
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use sea_orm::{DatabaseConnection, SqlxPostgresConnector, sqlx::postgres::PgPoolOptions};
use secrecy::{ExposeSecret, SecretString};
use time::Duration;
use tokio::net::TcpListener;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{RedisStore, fred::prelude::*};

type Server = Serve<TcpListener, IntoMakeService<Router>, Router>;

#[derive(Debug, Clone)]
pub struct HmacSecret(pub SecretString);

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> anyhow::Result<Self, anyhow::Error> {
        let db_connection = get_db_connection(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = std::net::TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;

        let port = listener.local_addr()?.port();

        let server = run(
            listener,
            db_connection,
            email_client,
            format!(
                "{}:{}",
                configuration.application.base_url, configuration.application.port
            ),
            HmacSecret(configuration.application.hmac_secret),
            configuration.redis_uri,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn run(
    listener: std::net::TcpListener,
    db_connection: DatabaseConnection,
    email_client: EmailClient,
    base_url: String,
    secret: HmacSecret,
    redis_uri: SecretString,
) -> Result<Server, anyhow::Error> {
    let app_state = AppState {
        db_connection,
        email_client,
        base_url,
        secret,
    };

    let redis_pool = Pool::new(
        Config::from_url(redis_uri.expose_secret())?,
        None,
        None,
        None,
        6,
    )
    .unwrap();
    let _redis_conn = redis_pool.connect();
    redis_pool.wait_for_connect().await?;
    let redis_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(redis_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(10)));

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .route("/", get(home))
        .route("/login", get(login_form).post(login))
        .route("/index", get(index))
        .route("/{name}", get(greet))
        .route("/admin/dashboard", get(admin_dashboard))
        .route("/admin/password", get(change_password_form))
        .route("/admin/password", post(change_password))
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .layer(MessagesManagerLayer)
        .layer(session_layer)
        .with_state(app_state.clone());

    let listener = TcpListener::from_std(listener)?;
    println!(
        "Listening on http://{:?}",
        listener.local_addr().expect("network error")
    );

    let server = axum::serve(listener, app.into_make_service());
    Ok(server)
}

pub async fn build(configuration: Settings) -> Result<Server, anyhow::Error> {
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = std::net::TcpListener::bind(address).expect("Failed to bind random port");
    let _ = listener.set_nonblocking(true);

    let configuration = get_configuration().expect("Failed to read configuration");
    let db_connection = get_db_connection(&configuration.database);

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    run(
        listener,
        db_connection,
        email_client,
        configuration.application.base_url,
        HmacSecret(configuration.application.hmac_secret),
        configuration.redis_uri,
    )
    .await
}

pub fn get_db_connection(configuration: &DatabaseSettings) -> DatabaseConnection {
    let db_pool = PgPoolOptions::new().connect_lazy_with(configuration.with_db());
    SqlxPostgresConnector::from_sqlx_postgres_pool(db_pool)
}
