use actix_web::{App, HttpServer};
use zero2prod::app_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(app_config))
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
