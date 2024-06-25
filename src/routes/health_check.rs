use actix_web::{Responder, HttpResponse};

pub async fn health() -> impl Responder {
    HttpResponse::Ok()
}
