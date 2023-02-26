mod login;
mod home;
mod newsletters;
mod subscriptions;
mod subscriptions_confirm;

use super::domain::SubscriberEmail;
use super::EmailClient;
use actix_web::{HttpResponse, Responder};

pub use home::home;
pub use login::login;
pub use newsletters::post_newsletter;
pub use subscriptions::{subscribe, ApplicationBaseUrl};
pub use subscriptions_confirm::confirm_subscription;

#[tracing::instrument]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().await
}
