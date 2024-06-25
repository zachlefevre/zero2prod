use std::net::TcpListener;

use crate::routes::*;
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let http_server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
//            .wrap(Logger::default())
            .route("/health", web::get().to(health))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(connection.clone())
    });
    Ok(http_server.listen(listener)?.run())
}
