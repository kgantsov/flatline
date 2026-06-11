use crate::notify::{NotificationEvent, Notifier};
use anyhow::Result;
use axum::async_trait;
use reqwest::Client;
use serde::Serialize;
use tracing::info;

#[derive(Debug, Clone, Serialize)]
struct SlackMessage {
    text: String,
}

pub struct SlackNotifier {
    client: Client,
    url: String,
}

impl SlackNotifier {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }
}

#[async_trait]
impl Notifier for SlackNotifier {
    async fn send(&self, event: NotificationEvent) -> Result<()> {
        info!("Sending Slack notification for event: {:?}", event);

        let message = match event {
            NotificationEvent::MonitorDown {
                monitor,
                checked_at,
                error,
            } => SlackMessage {
                text: format!(
                    "🚨 Monitor *{}* is down as of {}. Error: {}",
                    monitor.name, checked_at, error
                ),
            },
            NotificationEvent::MonitorRecovered { monitor, incident } => SlackMessage {
                text: format!(
                    "✅ Monitor *{}* has recovered. Incident ID: {}",
                    monitor.name, incident.id
                ),
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
