use actix_web::{web, HttpResponse, Responder};
use rand::Rng;
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

    let subscriber_id = match insert_subscriber(db_pool.get_ref(), &subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(e) => {
            tracing::error!("Failed to insert subscriber: {e}");
            return HttpResponse::InternalServerError().await;
        }
    };

    let subscription_token = generate_subscription_token().await;
    let token_result = store_token(db_pool.get_ref(), &subscriber_id, &subscription_token).await;
    if let Err(e) = token_result {
        tracing::error!("Failed to store token: {e}");
        return HttpResponse::InternalServerError().await;
    }

    let email_result = send_confirmation_email(
        &email_client,
        &subscriber,
        &app_base_url,
        &subscription_token,
    )
    .await;
    if let Err(e) = email_result {
        tracing::error!("Failed to send email: {e}");
        return HttpResponse::InternalServerError().await;
    }

    HttpResponse::Ok().await
}

async fn generate_subscription_token() -> String {
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument]
async fn insert_subscriber(db_pool: &PgPool, subscriber: &Subscriber) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        id,
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool)
    .await?;
    Ok(id)
}

#[tracing::instrument]
async fn store_token(
    db_pool: &PgPool,
    subscriber_id: &Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
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
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let app_base_url = &app_base_url.0;
    let confirmation_link =
        format!("{app_base_url}/subscriptions/confirm?subscription_token={subscription_token}");
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
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let subscriber_id =
        match get_subscriber_id_from_token(&db_pool, &query.subscription_token).await {
            Ok(subscriber_id) => subscriber_id,
            Err(_) => return HttpResponse::InternalServerError().await,
        };
    match subscriber_id {
        None => HttpResponse::Unauthorized().await,
        Some(subscriber_id) => {
            if confirm_subscriber(&db_pool, subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().await;
            }
            HttpResponse::Ok().await
        }
    }
}

#[tracing::instrument]
pub async fn confirm_subscriber(db_pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument]
pub async fn get_subscriber_id_from_token(
    db_pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token,
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}
