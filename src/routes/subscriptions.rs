use actix_web::{http::StatusCode, web, HttpResponse, Responder, ResponseError};
use rand::Rng;
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{Subscriber, SubscriberName},
    EmailClient, SubscriberEmail,
};

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

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] Box<dyn std::error::Error>),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{e}\n")?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{cause}")?;
        current = cause.source();
    }
    Ok(())
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(skip(db_pool))]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    app_base_url: web::Data<ApplicationBaseUrl>,
) -> Result<impl Responder, SubscribeError> {
    let subscriber = Subscriber::try_from(form.0).map_err(SubscribeError::ValidationError)?;
    let mut transaction = db_pool
        .begin()
        .await
        .map_err(|e| SubscribeError::UnexpectedError(e.into()))?;
    let subscriber_id = insert_subscriber(&mut transaction, &subscriber)
        .await
        .map_err(|e| SubscribeError::UnexpectedError(e.into()))?;
    let subscription_token = generate_subscription_token().await;
    store_token(&mut transaction, &subscriber_id, &subscription_token)
        .await
        .map_err(|e| SubscribeError::UnexpectedError(e.into()))?;
    transaction
        .commit()
        .await
        .map_err(|e| SubscribeError::UnexpectedError(e.into()))?;
    send_confirmation_email(
        &email_client,
        &subscriber,
        &app_base_url,
        &subscription_token,
    )
    .await
    .map_err(|e| SubscribeError::UnexpectedError(e.into()))?;
    Ok(HttpResponse::Ok())
}

async fn generate_subscription_token() -> String {
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber: &Subscriber,
) -> Result<Uuid, sqlx::Error> {
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
    .execute(transaction)
    .await?;
    Ok(id)
}

#[tracing::instrument]
async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: &Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
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
