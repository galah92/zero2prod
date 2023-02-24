use super::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    auth_token: String,
    from: SubscriberEmail,
}

#[derive(serde::Serialize)]
struct EmailRequestBody<'a> {
    personalizations: Vec<Personalization<'a>>,
    from: EmailAddress<'a>,
    content: Vec<Content<'a>>,
}

#[derive(serde::Serialize)]
struct Personalization<'a> {
    to: Vec<EmailAddress<'a>>,
    subject: &'a str,
}

#[derive(serde::Serialize)]
struct EmailAddress<'a> {
    email: &'a str,
}

#[derive(serde::Serialize)]
struct Content<'a> {
    #[serde(rename = "type")]
    content_type: &'a str,
    value: &'a str,
}

impl EmailClient {
    pub fn new(base_url: String, auth_token: String, from: SubscriberEmail) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();
        Self {
            http_client,
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
    ) -> Result<(), reqwest::Error> {
        // based on https://docs.sendgrid.com/api-reference/mail-send/mail-send#body
        let body = EmailRequestBody {
            personalizations: vec![Personalization {
                to: vec![EmailAddress { email: to.as_ref() }],
                subject,
            }],
            from: EmailAddress {
                email: self.from.as_ref(),
            },
            content: vec![
                Content {
                    content_type: "text/plain",
                    value: text_content,
                },
                Content {
                    content_type: "text/html",
                    value: html_content,
                },
            ],
        };
        self.http_client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
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

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        let auth_token = Faker.fake();
        let from = SubscriberEmail::parse(SafeEmail().fake())?;
        let email_client = EmailClient::new(mock_server.uri(), auth_token, from);

        let to = SubscriberEmail::parse(SafeEmail().fake())?;
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(202))
            .expect(1)
            .mount(&mock_server)
            .await;

        email_client
            .send_email(&to, &subject, &content, &content)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() -> Result<(), Box<dyn std::error::Error>>
    {
        // Arrange
        let mock_server = MockServer::start().await;
        let auth_token = Faker.fake();
        let from = SubscriberEmail::parse(SafeEmail().fake())?;
        let email_client = EmailClient::new(mock_server.uri(), auth_token, from);

        let to = SubscriberEmail::parse(SafeEmail().fake())?;
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let res = email_client
            .send_email(&to, &subject, &content, &content)
            .await;
        assert!(res.is_err());

        Ok(())
    }
}
