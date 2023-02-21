mod settings;
mod telemetry;

pub use settings::get_settings;
pub use telemetry::init_tracing;

use actix_web::web;

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/health_check", web::get().to(routes::health_check));
    cfg.route("/subscriptions", web::post().to(routes::subscribe));
}

mod routes {
    use actix_web::{web, HttpResponse, Responder};
    use serde::Deserialize;
    use sqlx::PgPool;
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[tracing::instrument]
    pub async fn health_check() -> impl Responder {
        HttpResponse::Ok().await
    }

    #[derive(Deserialize, Debug)]
    pub struct FormData {
        name: String,
        email: String,
    }

    // #[post("/subscriptions")]
    #[tracing::instrument(skip(db_pool))]
    pub async fn subscribe(
        form: web::Form<FormData>,
        db_pool: web::Data<PgPool>,
    ) -> impl Responder {
        let query_result = sqlx::query!(
            r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
            "#,
            Uuid::new_v4(),
            form.email,
            form.name,
            OffsetDateTime::now_utc(),
        )
        .execute(db_pool.get_ref())
        .await;
        match query_result {
            Ok(_) => HttpResponse::Ok(),
            Err(e) => {
                tracing::error!("{}", e);
                HttpResponse::InternalServerError()
            }
        }
        .await
    }
}
