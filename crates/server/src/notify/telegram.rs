use crate::notify::{NotificationEvent, Notifier};
use anyhow::Result;
use axum::async_trait;
use reqwest::Client;
use serde::Serialize;
use tracing::info;

#[derive(Debug, Clone, Serialize)]
struct TelegramMessage {
    chat_id: String,
    text: String,
    disable_notification: bool,
}

pub struct TelegramNotifier {
    client: Client,
    url: String,
    chat_id: String,
}

impl TelegramNotifier {
    pub fn new(url: String, chat_id: String) -> Self {
        Self {
            client: Client::new(),
            url,
            chat_id,
        }
    }
}

#[async_trait]
impl Notifier for TelegramNotifier {
    async fn send(&self, event: NotificationEvent) -> Result<()> {
        info!("Sending Slack notification for event: {:?}", event);

        let message = match event {
            NotificationEvent::MonitorDown {
                monitor,
                checked_at,
                error,
            } => TelegramMessage {
                text: format!(
                    "🚨 Monitor *{}* is down as of {}. Error: {}",
                    monitor.name, checked_at, error
                ),
                chat_id: self.chat_id.clone(),
                disable_notification: true,
            },
            NotificationEvent::MonitorRecovered { monitor, incident } => TelegramMessage {
                text: format!(
                    "✅ Monitor *{}* has recovered. Incident ID: {}",
                    monitor.name, incident.id
                ),
                chat_id: self.chat_id.clone(),
                disable_notification: true,
            },
        };

        self.client
            .post(&self.url)
            .json(&message)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
