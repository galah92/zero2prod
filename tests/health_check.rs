use actix_web::{http, test, App};
use zero2prod::app_config;

#[actix_web::test]
async fn health_check_works() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/health_check").to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);
}
