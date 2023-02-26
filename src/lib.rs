mod domain;
mod email;
mod routes;
mod settings;
mod telemetry;

pub use domain::SubscriberEmail;
pub use email::EmailClient;
pub use settings::get_settings;
pub use telemetry::init_tracing;

use actix_web::web;

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/health_check", web::get().to(routes::health_check));
    cfg.route("/subscriptions", web::post().to(routes::subscribe));
    cfg.route(
        "/subscriptions/confirm",
        web::get().to(routes::confirm_subscription),
    );
}
