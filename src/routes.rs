use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::domain;

#[tracing::instrument]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().await
}

#[derive(Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(skip(db_pool))]
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    let name = domain::SubscriberName::parse(form.name.clone());
    let name = match name {
        Ok(name) => name,
        Err(_) => {
            tracing::warn!("Invalid name: {}", &form.name);
            return HttpResponse::BadRequest().await;
        }
    };
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        form.email,
        name.as_ref(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool.get_ref())
    .await
    .map_or_else(
        |e| {
            tracing::error!("{}", e);
            HttpResponse::InternalServerError()
        },
        |_| HttpResponse::Ok(),
    )
    .await
}
