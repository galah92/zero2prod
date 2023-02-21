use actix_web::{App, HttpServer};
use zero2prod::app_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = zero2prod::get_settings().expect("Failed to read config");
    let address = format!("127.0.0.1:{}", settings.port);

    HttpServer::new(|| App::new().configure(app_config))
        .bind(address)?
        .run()
        .await
}
