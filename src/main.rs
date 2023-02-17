use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health_check").route(web::get().to(health_check)));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(config))
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
