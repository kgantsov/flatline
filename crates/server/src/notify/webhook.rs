use crate::notify::{NotificationEvent, Notifier};
use anyhow::Result;
use axum::async_trait;
use reqwest::{Client, Method};
use shared::models::HttpMethod;

pub struct WebhookNotifier {
    client: Client,
    url: String,
    method: Method,
}

impl WebhookNotifier {
    pub fn new(method: Option<HttpMethod>, url: String) -> Self {
        Self {
            client: Client::new(),
            method: match method.unwrap_or(HttpMethod::Post) {
                HttpMethod::Get => Method::GET,
                HttpMethod::Post => Method::POST,
                HttpMethod::Put => Method::PUT,
                HttpMethod::Patch => Method::PATCH,
                HttpMethod::Delete => Method::DELETE,
                HttpMethod::Head => Method::HEAD,
                HttpMethod::Options => Method::OPTIONS,
            },
            url,
        }
    }
}

#[async_trait]
impl Notifier for WebhookNotifier {
    async fn send(&self, event: NotificationEvent) -> Result<()> {
        self.client
            .request(self.method.clone(), &self.url)
            .json(&event)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
