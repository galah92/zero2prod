use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::domain::{Subscriber, SubscriberEmail, SubscriberName};

#[tracing::instrument]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().await
}

#[derive(Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for Subscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { name, email })
    }
}

#[tracing::instrument(skip(db_pool))]
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    let subscriber = Subscriber::try_from(form.0);
    let subscriber = match subscriber {
        Ok(subscriber) => subscriber,
        Err(e) => {
            tracing::warn!("{e}");
            return HttpResponse::BadRequest().await;
        }
    };
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
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
