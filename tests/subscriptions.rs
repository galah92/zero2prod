use actix_web::{
    http::{self, header::ContentType},
    test, App,
};
use zero2prod::app_config;

#[actix_web::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let req = test::TestRequest::post()
        .uri("/subscriptions")
        .insert_header(ContentType::form_url_encoded())
        .set_payload(body)
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let req = test::TestRequest::post()
            .uri("/subscriptions")
            .insert_header(ContentType::form_url_encoded())
            .set_payload(invalid_body)
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(
            res.status(),
            http::StatusCode::BAD_REQUEST,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
