use actix_web::{
    http::{self, header::ContentType},
    test, web, App,
};
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
        .mount(&mock_server)
        .await;

    (email_client, mock_server)
}

#[sqlx::test]
async fn confirmations_without_token_are_rejected_with_a_400(
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

    let req = test::TestRequest::get()
        .uri("/subscriptions/confirm")
        .to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[sqlx::test]
async fn the_link_returned_by_subscribe_returned_a_200_if_called(
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
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(body)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
    let confirmation_link = links[0].as_str();
    let link_uri = confirmation_link
        .split('/')
        .skip(3)
        .collect::<Vec<_>>()
        .join("/");
    let link_uri = format!("/{link_uri}");

    let req = test::TestRequest::get().uri(&link_uri).to_request();
    let res = test::call_service(&app, req).await;

    assert_eq!(res.status(), http::StatusCode::OK);

    Ok(())
}
