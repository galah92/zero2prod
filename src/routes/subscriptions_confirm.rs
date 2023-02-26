use actix_web::{http::StatusCode, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct ConfirmationQuery {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmSubscriptionError {
    #[error("{0}")]
    Unauthorized(String),
    #[error(transparent)]
    UnexpectedError(#[from] Box<dyn std::error::Error>),
}

impl std::fmt::Debug for ConfirmSubscriptionError {
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

impl ResponseError for ConfirmSubscriptionError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmSubscriptionError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ConfirmSubscriptionError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument]
pub async fn confirm_subscription(
    query: web::Query<ConfirmationQuery>,
    db_pool: web::Data<PgPool>,
) -> Result<impl Responder, ConfirmSubscriptionError> {
    let subscriber_id = get_subscriber_id_from_token(&db_pool, &query.subscription_token)
        .await
        .map_err(|e| ConfirmSubscriptionError::UnexpectedError(e.into()))?;
    subscriber_id.ok_or_else(|| {
        ConfirmSubscriptionError::Unauthorized("Invalid subscription token".into())
    })?;
    confirm_subscriber(&db_pool, subscriber_id.unwrap())
        .await
        .map_err(|e| ConfirmSubscriptionError::UnexpectedError(e.into()))?;
    Ok(HttpResponse::Ok())
}

#[tracing::instrument]
pub async fn confirm_subscriber(db_pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(db_pool)
    .await?;
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
    .await?;
    Ok(result.map(|r| r.subscriber_id))
}
