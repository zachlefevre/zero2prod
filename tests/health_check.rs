use std::net::TcpListener;

use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::get_configuration,
    startup,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> =
    Lazy::new(|| {
        if std::env::var("TEST_LOG").is_ok() {
            init_subscriber(get_subscriber("test".into(), "debug".into(), std::io::stdout))
        } else {
            init_subscriber(get_subscriber("test".into(), "debug".into(), std::io::sink))
        }

    });

#[tokio::test]
async fn subscribe_returns_a_400_for_missing_data() {
    let TestApp { address, .. } = spawn_app().await;

    let client = reqwest::Client::new();

    let cases = vec![
        ("name=ursula", "missing email"),
        ("email=ursula", "missing name"),
        ("", "missing both"),
    ];

    for (request, reason) in cases {
        let response = client
            .post(format!("{}/subscriptions", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(request)
            .send()
            .await
            .unwrap();

        assert_eq!(
            400,
            response.status(),
            "The API did not fail witha 400 when the payload was {}.",
            reason
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let TestApp { address, db_pool } = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("name=le%20guin&email=ursula_le_guin%40gmail.com")
        .send()
        .await
        .unwrap();

    assert_eq!(200, response.status());
    let saved = sqlx::query!("SELECT email, name from subscriptions")
        .fetch_one(&db_pool)
        .await
        .unwrap();
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn health_check_works() {
    let TestApp { address, .. } = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health", address))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let mut config = get_configuration().unwrap();
    config.database.database_name = Uuid::new_v4().to_string();
    let pool = configure_database(config.database).await;

    let server = startup::run(listener, pool.clone()).unwrap();
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}

async fn configure_database(config: zero2prod::configuration::DatabaseSettings) -> PgPool {
    PgConnection::connect(&config.connection_string_no_name().expose_secret())
        .await
        .unwrap()
        .execute(format!(r#"create database "{}";"#, config.database_name).as_str())
        .await
        .unwrap();

    let pool = PgPool::connect(&config.connection_string().expose_secret()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
