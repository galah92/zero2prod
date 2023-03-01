mod domain;
mod email;
mod routes;
mod settings;
mod telemetry;

pub use domain::SubscriberEmail;
pub use email::EmailClient;
pub use routes::ApplicationBaseUrl;
pub use settings::*;
pub use telemetry::init_tracing;

use actix_web::web::{get, post, ServiceConfig};
use routes::*;

pub fn app_config(cfg: &mut ServiceConfig) {
    cfg.route("/health_check", get().to(health_check));
    cfg.route("/subscriptions", post().to(subscribe));
    cfg.route("/subscriptions/confirm", get().to(confirm_subscription));
    cfg.route("/newsletters", post().to(post_newsletter));
    cfg.route("/", get().to(home));
    cfg.route("/login", get().to(login_page));
    cfg.route("/login", post().to(login));
}
