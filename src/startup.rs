use crate::{
    configuration::{DatabaseSettings, Settings, get_configuration},
    email_client::EmailClient,
    routes::{AppState, confirm, health_check::*, subscribe},
};
use axum::{
    Router,
    routing::{IntoMakeService, get, post},
    serve::Serve,
};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use sea_orm::{DatabaseConnection, SqlxPostgresConnector, sqlx::postgres::PgPoolOptions};
use tokio::net::TcpListener;

type Server = Serve<TcpListener, IntoMakeService<Router>, Router>;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
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
        listener.set_nonblocking(true).unwrap();

        let port = listener.local_addr().unwrap().port();

        let server = run(
            listener,
            db_connection,
            email_client,
            configuration.application.base_url,
        )?;

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

pub fn run(
    listener: std::net::TcpListener,
    db_connection: DatabaseConnection,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    let app_state = AppState {
        db_connection,
        email_client,
        base_url,
    };
    let app = Router::new()
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .route("/", get(index))
        .route("/{name}", get(greet))
        .with_state(app_state.clone());

    let listener = TcpListener::from_std(listener)?;
    println!(
        "Listening on http://{:?}",
        listener.local_addr().expect("network error")
    );

    let server = axum::serve(listener, app.into_make_service());
    Ok(server)
}

pub async fn build(configuration: Settings) -> Result<Server, std::io::Error> {
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
    )
}

pub fn get_db_connection(configuration: &DatabaseSettings) -> DatabaseConnection {
    let db_pool = PgPoolOptions::new().connect_lazy_with(configuration.with_db());
    SqlxPostgresConnector::from_sqlx_postgres_pool(db_pool)
}
