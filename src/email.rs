use super::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    _http_client: Client,
    _base_url: String,
    _sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            _http_client: Client::new(),
            _base_url: base_url,
            _sender: sender,
        }
    }

    pub async fn send_email(
        &self,
        _recipient: &SubscriberEmail,
        _subject: &str,
        _html_content: &str,
        _text_content: &str,
    ) -> Result<(), String> {
        self._http_client
            .post(&self._base_url)
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
    use fake::{faker::internet::en::SafeEmail, Fake};
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake())?;
        let email_client = EmailClient::new(mock_server.uri(), sender);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake())?;
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await?;

        Ok(())
    }
}
