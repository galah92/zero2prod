use actix_web::{get, web, HttpResponse, Responder};

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
}
