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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http;
    use actix_web::test;

    #[actix_web::test]
    async fn health_check_works() {
        let app = test::init_service(App::new().configure(config)).await;
        let req = test::TestRequest::get().uri("/health_check").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), http::StatusCode::OK);
    }
}
