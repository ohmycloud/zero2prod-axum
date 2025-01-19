use std::net::TcpListener;

use sea_orm::Database;
use secrecy::ExposeSecret;
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
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind random port");
    let _ = listener.set_nonblocking(true);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let connection = Database::connect(&connection_string.expose_secret().to_owned())
        .await
        .expect("Failed to connect to Postgres.");
    run(listener, connection)?.await
}
