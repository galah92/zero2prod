mod settings;
mod telemetry;

pub use settings::get_settings;
pub use telemetry::init_tracing;

use actix_web::web;

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/health_check", web::get().to(routes::health_check));
    cfg.route("/subscriptions", web::post().to(routes::subscribe));
}

mod routes {
    use actix_web::{web, HttpResponse, Responder};
    use serde::Deserialize;
    use sqlx::PgPool;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::domain;

    #[tracing::instrument]
    pub async fn health_check() -> impl Responder {
        HttpResponse::Ok().await
    }

    #[derive(Deserialize, Debug)]
    pub struct FormData {
        name: String,
        email: String,
    }

    #[tracing::instrument(skip(db_pool))]
    pub async fn subscribe(
        form: web::Form<FormData>,
        db_pool: web::Data<PgPool>,
    ) -> impl Responder {
        let name = domain::SubscriberName::parse(form.name.clone());
        let name = match name {
            Ok(name) => name,
            Err(_) => {
                tracing::warn!("Invalid name: {}", &form.name);
                return HttpResponse::BadRequest().await;
            }
        };
        sqlx::query!(
            r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
            "#,
            Uuid::new_v4(),
            form.email,
            name.as_ref(),
            OffsetDateTime::now_utc(),
        )
        .execute(db_pool.get_ref())
        .await
        .map_or_else(
            |e| {
                tracing::error!("{}", e);
                HttpResponse::InternalServerError()
            },
            |_| HttpResponse::Ok(),
        )
        .await
    }
}

mod domain {
    use unicode_segmentation::UnicodeSegmentation;

    pub struct SubscriberName(String);

    impl SubscriberName {
        pub fn parse(s: String) -> Result<Self, String> {
            let is_empty_or_whitespace = s.trim().is_empty();
            let is_too_long = s.graphemes(true).count() > 256;

            let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
            let contains_forbidden_chars = s.chars().any(|c| forbidden_chars.contains(&c));

            if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
                Err(format!("Invalid subscriber name: {s}"))
            } else {
                Ok(Self(s))
            }
        }
    }

    impl AsRef<str> for SubscriberName {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    #[cfg(test)]
    mod tests {
        use super::SubscriberName;

        #[test]
        fn a_256_grapheme_long_name_is_valid() {
            let name = "a".repeat(256);
            assert!(SubscriberName::parse(name).is_ok());
        }

        #[test]
        fn a_name_longer_than_256_graphemes_is_invalid() {
            let name = "a".repeat(257);
            assert!(SubscriberName::parse(name).is_err());
        }

        #[test]
        fn whitespace_only_names_are_rejected() {
            let name = " ".to_string();
            assert!(SubscriberName::parse(name).is_err());
        }

        #[test]
        fn empty_string_is_rejected() {
            let name = "".to_string();
            assert!(SubscriberName::parse(name).is_err());
        }
        #[test]
        fn names_containing_an_invalid_character_are_rejected() {
            for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
                let name = name.to_string();
                assert!(SubscriberName::parse(name).is_err());
            }
        }
        #[test]
        fn a_valid_name_is_parsed_successfully() {
            let name = "Ursula Le Guin".to_string();
            assert!(SubscriberName::parse(name).is_ok());
        }
    }
}
