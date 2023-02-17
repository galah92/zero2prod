use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;

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
async fn subscribe(form: web::Form<FormData>) -> impl Responder {
    format!("Welcome, {} <{}>!", form.name, form.email)
}

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check).service(subscribe);
}
