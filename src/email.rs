use super::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    auth_token: String,
    from: SubscriberEmail,
}

#[derive(serde::Serialize)]
struct EmailRequestBody {
    personalizations: Vec<Personalization>,
    from: EmailAddress,
    content: Vec<Content>,
}

#[derive(serde::Serialize)]
struct Personalization {
    to: Vec<EmailAddress>,
    subject: String,
}

#[derive(serde::Serialize)]
struct EmailAddress {
    email: String,
}

#[derive(serde::Serialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    value: String,
}

impl EmailClient {
    pub fn new(base_url: String, auth_token: String, from: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            auth_token,
            from,
        }
    }

    pub async fn send_email(
        &self,
        to: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        // based on https://docs.sendgrid.com/api-reference/mail-send/mail-send#body
        let body = EmailRequestBody {
            personalizations: vec![Personalization {
                to: vec![EmailAddress {
                    email: to.as_ref().to_string(),
                }],
                subject: subject.to_string(),
            }],
            from: EmailAddress {
                email: self.from.as_ref().to_string(),
            },
            content: vec![
                Content {
                    content_type: "text/plain".to_string(),
                    value: text_content.to_string(),
                },
                Content {
                    content_type: "text/html".to_string(),
                    value: html_content.to_string(),
                },
            ],
        };
        self.http_client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&body)
            .send()
            .await
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{EmailClient, SubscriberEmail};
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{faker::internet::en::SafeEmail, Fake, Faker};
    use wiremock::matchers::{any, header, header_exists, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_tobase_url() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        let auth_token = Faker.fake();
        let from = SubscriberEmail::parse(SafeEmail().fake())?;
        let email_client = EmailClient::new(mock_server.uri(), auth_token, from);

        Mock::given(any())
            .and(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let to = SubscriberEmail::parse(SafeEmail().fake())?;
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        email_client
            .send_email(&to, &subject, &content, &content)
            .await?;
        Ok(())
    }
}
