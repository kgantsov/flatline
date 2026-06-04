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
    /// Whether the monitor is actively running checks.
    #[schema(example = true)]
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
