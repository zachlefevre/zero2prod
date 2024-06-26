use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}

#[tracing::instrument(name = "saving new subscriber details to DB", skip(connection, form))]
pub async fn insert_subscriber(form: &SubscribeFormData, connection: &PgPool) -> Result<(), sqlx::Error>  {
    sqlx::query!(
        r#"insert into subscriptions (id, email, name, subscribed_at) values ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
        .execute(connection)
        .await.map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}

#[tracing::instrument(name = "adding a subscriber", skip(form, connection), fields(subscriber_email = %form.email,
                                                                                   subscriber_name = %form.name))]
pub async fn subscribe(
    connection: web::Data<PgPool>,
    form: web::Form<SubscribeFormData>,
) -> impl Responder {
    match insert_subscriber(&form, &connection).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    }
}
