use actix_web::{web, HttpResponse, Responder};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health_check").route(web::get().to(health_check)));
}
