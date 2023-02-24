use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use zero2prod::{app_config, get_settings, EmailClient, SubscriberEmail};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    zero2prod::init_tracing();

    let settings = get_settings().expect("Failed to read config");
    let address = format!("{}:{}", settings.app_host, settings.app_port);

    let db_pool = PgPool::connect_lazy(&settings.database_url).unwrap();
    let db_pool = web::Data::new(db_pool);

    let email_client = EmailClient::new(
        settings.email_base_url,
        settings.email_auth_token,
        SubscriberEmail::parse(settings.email_sender).unwrap(),
    );
    let email_client = web::Data::new(email_client);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(app_config)
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .bind(address)?
    .run()
    .await
}
