use actix_web::{
    http::{self, header::ContentType},
    test, web, App,
};
use fake::{faker::internet::en::SafeEmail, Fake, Faker};
use sqlx::PgPool;
use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};
use zero2prod::{app_config, EmailClient, SubscriberEmail};

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
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client)),
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

    let record = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&db_pool)
        .await?;
    assert_eq!(record.email, "ursula_le_guin@gmail.com");
    assert_eq!(record.name, "le guin");

    Ok(())
}

#[sqlx::test]
async fn subscribe_returns_a_400_when_data_is_missing(
    db_pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (email_client, _) = get_mock_client().await;
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client)),
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
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client)),
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
    let app = test::init_service(
        App::new()
            .configure(app_config)
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(email_client)),
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
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(body)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
    assert_eq!(links.len(), 2); // one for text, one for html

    Ok(())
}
