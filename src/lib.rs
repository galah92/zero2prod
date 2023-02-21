mod settings;

pub use settings::get_settings;

use actix_web::web;
pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(routes::health_check).service(routes::subscribe);
}

mod routes {
    use actix_web::{get, post, web, HttpResponse, Responder};
    use serde::Deserialize;
    use sqlx::PgPool;
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[get("/health_check")]
    async fn health_check() -> impl Responder {
        HttpResponse::Ok()
    }

    #[derive(Deserialize)]
    struct FormData {
        name: String,
        email: String,
    }

    #[post("/subscriptions")]
    async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
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
                println!("Error: {}", e);
                HttpResponse::InternalServerError()
            }
        }
    }
}
