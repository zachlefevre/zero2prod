use std::{io::Error, net::TcpListener};

use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, startup::run, telemetry::{self, get_subscriber}};

#[tokio::main]
async fn main() -> Result<(), Error> {


    telemetry::init_subscriber(get_subscriber("zero2prod".into(), "info".into(), std::io::stdout));

    let configuration = get_configuration().expect("BAH!");
    let listener =
        TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port)).unwrap();

    let connection_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
            .await
            .unwrap();

    run(
        listener,
        connection_pool,
    )?
    .await
}
