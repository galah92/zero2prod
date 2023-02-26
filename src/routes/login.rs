use actix_web::{
    http::header::{ContentType, LOCATION},
    HttpResponse, Responder,
};

pub async fn login_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("login.html"))
}

pub async fn login() -> impl Responder {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/")) // redict to home page
        .await
}
