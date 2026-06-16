use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// HTTP method for an HTTP monitor.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

/// Protocol-specific monitor configuration.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MonitorConfig {
    Http {
        /// URL to poll.
        #[schema(example = "https://api.example.com/health")]
        url: String,
        /// HTTP method. Defaults to GET.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        method: Option<HttpMethod>,
        /// Expected HTTP status codes. Defaults to any 2xx.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_status: Option<Vec<u16>>,
    },
}

/// Status of a single monitor check.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MonitorCheckStatus {
    Up,
    Down,
}

impl std::fmt::Display for MonitorCheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorCheckStatus::Up => write!(f, "up"),
            MonitorCheckStatus::Down => write!(f, "down"),
        }
    }
}

/// A single recorded check result for a monitor.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitorCheck {
    /// Unique identifier.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// ID of the monitor that produced this check.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub monitor_id: Uuid,
    /// Whether the monitor was up or down.
    pub status: MonitorCheckStatus,
    /// HTTP status code returned, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    /// How long the request took, in milliseconds.
    #[schema(example = 142)]
    pub response_time_ms: u64,
    /// Error message if the check failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub checked_at: DateTime<Utc>,
}

/// A period during which a monitor was down.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Incident {
    /// Unique identifier.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// ID of the monitor this incident belongs to.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub monitor_id: Uuid,
    /// When the monitor first went down.
    pub started_at: DateTime<Utc>,
    /// When the monitor recovered. None if the incident is still open.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Notification channel delivery configuration.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NotificationChannelConfig {
    /// Send a POST request to a URL with a JSON payload.
    Webhook {
        /// Webhook URL to POST to.
        #[schema(example = "https://example.com/webhook")]
        url: String,
    },
    /// Send a message to a Slack channel via an incoming webhook.
    Slack {
        /// Slack incoming webhook URL.
        #[schema(example = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL")]
        webhook_url: String,
    },
}

/// A configured notification channel.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NotificationChannel {
    /// Unique identifier.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// Human-readable name.
    #[schema(example = "Ops Slack")]
    pub name: String,
    /// Delivery configuration.
    pub config: NotificationChannelConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A link between a monitor and a notification channel.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MonitorNotification {
    /// Unique identifier.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// ID of the monitor.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub monitor_id: Uuid,
    /// ID of the notification channel.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub channel_id: Uuid,
    /// Whether to also send a notification when the monitor recovers.
    #[schema(example = true)]
    pub on_recovery: bool,
    pub created_at: DateTime<Utc>,
}

/// A configured uptime monitor.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Monitor {
    /// Unique identifier.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// Human-readable name.
    #[schema(example = "Production API")]
    pub name: String,
    /// Protocol-specific configuration.
    pub config: MonitorConfig,
    /// How often to poll, in seconds.
    #[schema(example = 60)]
    pub interval: u32,
    /// Per-request timeout, in seconds.
    #[schema(example = 10)]
    pub timeout: u32,
    /// Number of retries before marking as down.
    #[schema(example = 3)]
    pub retries: u32,
    /// Whether the monitor is actively running checks.
    #[schema(example = true)]
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Latency percentiles for a monitor over a recent time period.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LatencyPercentiles {
    pub p50_ms: u64,
    pub p95_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Default, Clone)]
pub struct MonitorStats {
    pub uptime_7d: f64, // 0.0–1.0
    pub uptime_30d: f64,
    pub uptime_90d: f64,
    // latency percentiles in ms
    pub p50_7d: u64,
    pub p95_7d: u64,
    pub p50_30d: u64,
    pub p95_30d: u64,
    pub p50_90d: u64,
    pub p95_90d: u64,
}
