use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::EmailClient;

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
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> impl Responder {
    let subscriber = Subscriber::try_from(form.0);
    let subscriber = match subscriber {
        Ok(subscriber) => subscriber,
        Err(e) => {
            tracing::warn!("{e}");
            return HttpResponse::BadRequest().await;
        }
    };

    let query_result = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'confirmed')
            "#,
        Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool.get_ref())
    .await;
    if let Err(e) = query_result {
        tracing::error!("Failed to execute query: {e}");
        return HttpResponse::InternalServerError().await;
    }

    let confirmation_link = "https://my-api.com/subscriptions/confirm";
    let email_result = email_client
        .send_email(
            &subscriber.email,
            "Welcome!",
            &format!(
                "Welcome to our newsletter!<br />\
                Click <a href=\"{confirmation_link}\">here</a> to confirm your subscription."
            ),
            &format!(
                "Welcome to our newsletter!\nVisit {confirmation_link} to confirm your subscription."
            ),
        )
        .await;
    if let Err(e) = email_result {
        tracing::error!("Failed to send email: {e}");
        return HttpResponse::InternalServerError().await;
    }

    HttpResponse::Ok().await
}
