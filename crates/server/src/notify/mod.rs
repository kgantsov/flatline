pub mod slack;
pub mod telegram;
pub mod webhook;

use anyhow::Result;
use axum::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use shared::models::{Incident, Monitor};

#[derive(Debug, Clone, Serialize)]
pub enum NotificationEvent {
    MonitorDown {
        monitor: Monitor,
        checked_at: DateTime<Utc>,
        error: String,
    },
    MonitorRecovered {
        monitor: Monitor,
        incident: Incident,
    },
}

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send(&self, event: NotificationEvent) -> Result<()>;
}
