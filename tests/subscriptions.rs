mod common;

use actix_web::{
    http::{self, header::ContentType},
    test, web, App,
};
use fake::{faker::internet::en::SafeEmail, Fake, Faker};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};
use zero2prod::{app_config, ApplicationBaseUrl, EmailClient, SubscriberEmail};

async fn get_mock_client() -> (EmailClient, MockServer) {
    let mock_server = MockServer::start().await;

    let auth_token = Faker.fake();
    let from = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
    let email_client = EmailClient::new(mock_server.uri(), auth_token, from);

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    (email_client, mock_server)
}

#[sqlx::test]
async fn subscribe_returns_a_200_for_valid_form_data(
    db_pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (email_client, _) = get_mock_client().await;
    let app_base_url = ApplicationBaseUrl("http://127.0.0.1".to_string());
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client))
            .app_data(web::Data::new(app_base_url)),
    )
    .await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let req = test::TestRequest::post()
        .uri("/subscriptions")
        .insert_header(ContentType::form_url_encoded())
        .set_payload(body)
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    Ok(())
}

#[sqlx::test]
async fn subscribe_persists_the_new_subscriber(
    db_pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (email_client, _) = get_mock_client().await;
    let app_base_url = ApplicationBaseUrl("http://127.0.0.1".to_string());
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client))
            .app_data(web::Data::new(app_base_url)),
    )
    .await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let req = test::TestRequest::post()
        .uri("/subscriptions")
        .insert_header(ContentType::form_url_encoded())
        .set_payload(body)
        .to_request();
    test::call_service(&app, req).await;

    let record = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&db_pool)
        .await?;
    assert_eq!(record.email, "ursula_le_guin@gmail.com");
    assert_eq!(record.name, "le guin");
    assert_eq!(record.status, "pending_confirmation");

    Ok(())
}

#[sqlx::test]
async fn subscribe_returns_a_400_when_data_is_missing(
    db_pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (email_client, _) = get_mock_client().await;
    let app_base_url = ApplicationBaseUrl("http://127.0.0.1".to_string());
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client))
            .app_data(web::Data::new(app_base_url)),
    )
    .await;

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
            "The API did not fail with 400 Bad Request when the payload was {error_message}."
        );
    }

    Ok(())
}

#[sqlx::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty(
    db_pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (email_client, _) = get_mock_client().await;
    let app_base_url = ApplicationBaseUrl("http://127.0.0.1".to_string());
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client))
            .app_data(web::Data::new(app_base_url)),
    )
    .await;

    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        let req = test::TestRequest::post()
            .uri("/subscriptions")
            .insert_header(ContentType::form_url_encoded())
            .set_payload(body)
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(
            res.status(),
            http::StatusCode::BAD_REQUEST,
            "The API did not fail with 400 BAD_REQUEST when the payload was {description}."
        );
    }

    Ok(())
}

#[sqlx::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data(
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

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let req = test::TestRequest::post()
        .uri("/subscriptions")
        .insert_header(ContentType::form_url_encoded())
        .set_payload(body)
        .to_request();
    test::call_service(&app, req).await;

    let email_request = &mock_server.received_requests().await.unwrap()[0];
    let body = std::str::from_utf8(&email_request.body)?;
    let confirmation_links = common::extract_links(body);
    assert_eq!(confirmation_links.len(), 2); // one for text, one for html

    Ok(())
}

#[sqlx::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error(
    db_pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    zero2prod::init_tracing();

    let (email_client, _) = get_mock_client().await;
    let app_base_url = ApplicationBaseUrl("http://127.0.0.1".to_string());
    let app = test::init_service(
        App::new()
            .wrap(TracingLogger::default())
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client))
            .app_data(web::Data::new(app_base_url)),
    )
    .await;

    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&db_pool)
        .await
        .unwrap();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let req = test::TestRequest::post()
        .uri("/subscriptions")
        .insert_header(ContentType::form_url_encoded())
        .set_payload(body)
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::INTERNAL_SERVER_ERROR);

    Ok(())
}
