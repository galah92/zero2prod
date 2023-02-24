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

    pub async fn send_email(&self) -> Result<(), String> {
        todo!()
    }
}
