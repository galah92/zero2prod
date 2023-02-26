use actix_web::{http::header::ContentType, HttpResponse};

pub async fn login() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("login.html"))
}
