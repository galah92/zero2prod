mod common;

use actix_web::{
    dev::Service,
    http::{self, header::ContentType},
    test, web, App,
};
use common::extract_links;
use fake::{faker::internet::en::SafeEmail, Fake, Faker};
use sqlx::PgPool;
use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};
use zero2prod::{app_config, ApplicationBaseUrl, EmailClient, SubscriberEmail};

async fn get_mock_client() -> (EmailClient, MockServer) {
    let mock_server = MockServer::start().await;

    let auth_token = Faker.fake();
    let from = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
    let email_client = EmailClient::new(mock_server.uri(), auth_token, from);

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .named("catch all")
        .mount(&mock_server)
        .await;

    (email_client, mock_server)
}

async fn create_unconfirmed_subscriber(
    app: &impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    mock_server: &MockServer,
) -> Vec<String> {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let req = test::TestRequest::post()
        .uri("/subscriptions")
        .insert_header(ContentType::form_url_encoded())
        .set_payload(body)
        .to_request();
    test::call_service(&app, req).await;

    let email_request = &mock_server.received_requests().await.unwrap()[0];
    let body = std::str::from_utf8(&email_request.body).unwrap();
    extract_links(body)
}

async fn create_confirmed_subscriber(
    app: &impl Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    mock_server: &MockServer,
) {
    let links = create_unconfirmed_subscriber(app, mock_server).await;
    let link_uri = &links[0].split('/').skip(3).collect::<Vec<_>>().join("/");
    let link_uri = format!("/{link_uri}");
    let req = test::TestRequest::get().uri(&link_uri).to_request();
    test::call_service(&app, req).await;
}

#[sqlx::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers(
    db_pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (email_client, mock_server) = get_mock_client().await;
    let app_base_url = ApplicationBaseUrl("http://127.0.0.1".to_string());
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client))
            .app_data(web::Data::new(app_base_url)),
    )
    .await;

    create_unconfirmed_subscriber(&app, &mock_server).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .named("email is not sent")
        .mount(&mock_server)
        .await;

    let body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    let req = test::TestRequest::post()
        .uri("/newsletters")
        .set_json(body)
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    Ok(())
}

#[sqlx::test]
#[ignore]
async fn newsletters_are_delivered_to_confirmed_subscribers(
    db_pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (email_client, mock_server) = get_mock_client().await;
    let app_base_url = ApplicationBaseUrl("http://127.0.0.1".to_string());
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client))
            .app_data(web::Data::new(app_base_url)),
    )
    .await;

    create_confirmed_subscriber(&app, &mock_server).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .named("email is sent")
        .expect(1)
        .mount(&mock_server)
        .await;

    let body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    let req = test::TestRequest::post()
        .uri("/newsletters")
        .set_json(body)
        .to_request();
    let res = test::call_service(&app, req).await;
    assert_eq!(res.status(), http::StatusCode::OK);

    Ok(())
}
