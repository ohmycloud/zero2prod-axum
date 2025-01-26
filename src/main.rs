use std::net::TcpListener;

use sea_orm::{SqlxPostgresConnector, sqlx::postgres::PgPoolOptions};
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // redirect all `log`'s events to our subscriber
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).expect("Failed to bind random port");
    let _ = listener.set_nonblocking(true);

    let configuration = get_configuration().expect("Failed to read configuration");
    let db_pool = PgPoolOptions::new().connect_lazy_with(configuration.database.with_db());
    let db_connection = SqlxPostgresConnector::from_sqlx_postgres_pool(db_pool);

    run(listener, db_connection)?.await
}
