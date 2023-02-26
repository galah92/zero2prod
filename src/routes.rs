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

#[derive(Debug)]
pub struct ApplicationBaseUrl(pub String);

#[tracing::instrument(skip(db_pool))]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    app_base_url: web::Data<ApplicationBaseUrl>,
) -> impl Responder {
    let subscriber = Subscriber::try_from(form.0);
    let subscriber = match subscriber {
        Ok(subscriber) => subscriber,
        Err(e) => {
            tracing::warn!("{e}");
            return HttpResponse::BadRequest().await;
        }
    };

    let query_result = insert_subscriber(db_pool.get_ref(), &subscriber).await;
    if let Err(e) = query_result {
        tracing::error!("Failed to execute query: {e}");
        return HttpResponse::InternalServerError().await;
    }

    let email_result = send_confirmation_email(&email_client, &subscriber, &app_base_url).await;
    if let Err(e) = email_result {
        tracing::error!("Failed to send email: {e}");
        return HttpResponse::InternalServerError().await;
    }

    HttpResponse::Ok().await
}

#[tracing::instrument]
async fn insert_subscriber(db_pool: &PgPool, subscriber: &Subscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool)
    .await?;
    Ok(())
}

#[tracing::instrument]
async fn send_confirmation_email(
    email_client: &EmailClient,
    subscriber: &Subscriber,
    app_base_url: &ApplicationBaseUrl,
) -> Result<(), reqwest::Error> {
    let app_base_url = &app_base_url.0;
    let confirmation_link =
        format!("{app_base_url}/subscriptions/confirm?subscription_token=mytoken");
    email_client
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
        .await?;
    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct ConfirmationQuery {
    subscription_token: String,
}

#[tracing::instrument]
pub async fn confirm_subscription(
    query: web::Query<ConfirmationQuery>,
    // db_pool: web::Data<PgPool>,
    // email_client: web::Data<EmailClient>,
) -> impl Responder {
    dbg!(query.0.subscription_token);
    HttpResponse::Ok().await
}
