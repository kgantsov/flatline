use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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
        #[schema(example = "GET")]
        method: Option<String>,
        /// Expected HTTP status codes. Defaults to any 2xx.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_status: Option<Vec<u16>>,
    },
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
