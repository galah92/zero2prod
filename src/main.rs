use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use zero2prod::app_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    zero2prod::init_tracing();

    let settings = zero2prod::get_settings().expect("Failed to read config");
    let address = format!("{}:{}", settings.app.host, settings.app.port);

    let db_pool = PgPool::connect(&settings.database.connection_string())
        .await
        .unwrap();
    let db_pool = web::Data::new(db_pool);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(app_config)
            .app_data(db_pool.clone())
    })
    .bind(address)?
    .run()
    .await
}
