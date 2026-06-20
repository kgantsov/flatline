use crate::notify::{NotificationEvent, Notifier};
use anyhow::Result;
use axum::async_trait;
use reqwest::Client;
use serde::Serialize;
use tracing::info;

#[derive(Debug, Clone, Serialize)]
struct DiscordMessage {
    embeds: Vec<Embed>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Embed {
    title: String,
    description: String,
    color: u32,
    timestamp: String,
}

pub struct DiscordNotifier {
    client: Client,
    url: String,
}

impl DiscordNotifier {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }
}

#[async_trait]
impl Notifier for DiscordNotifier {
    async fn send(&self, event: NotificationEvent) -> Result<()> {
        info!("Sending Discord notification for event: {:?}", event);

        let message = match event {
            NotificationEvent::MonitorDown {
                monitor,
                checked_at,
                error,
            } => DiscordMessage {
                embeds: vec![Embed {
                    title: format!("🚨 Monitor *{}* is down", monitor.name),
                    description: format!("Checked at: {}\nError: {}", checked_at, error),
                    color: 15158332,
                    timestamp: checked_at.to_rfc3339(),
                }],
            },
            NotificationEvent::MonitorRecovered { monitor, incident } => DiscordMessage {
                embeds: vec![Embed {
                    title: format!("✅ Monitor *{}* has recovered", monitor.name),
                    description: format!("Incident ID: {}", incident.id),
                    color: 5763719,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }],
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
