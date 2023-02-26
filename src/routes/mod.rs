mod home;
mod login;
mod newsletters;
mod subscriptions;
mod subscriptions_confirm;

use super::domain::SubscriberEmail;
use super::EmailClient;
use actix_web::{HttpResponse, Responder};

pub use home::*;
pub use login::*;
pub use newsletters::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;

#[tracing::instrument]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().await
}
