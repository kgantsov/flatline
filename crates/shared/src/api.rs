use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{MonitorCheckStatus, MonitorConfig, NotificationChannelConfig};

/// Request body for recording a new monitor check result.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMonitorCheckRequest {
    /// ID of the monitor this check belongs to.
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
}

/// Request body for updating an existing monitor. All fields are optional.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateMonitorRequest {
    /// Human-readable name for the monitor.
    #[schema(example = "Production API")]
    pub name: Option<String>,
    /// Protocol-specific configuration. Replaces the entire config when provided.
    pub config: Option<MonitorConfig>,
    /// Polling interval in seconds. Minimum 10.
    #[schema(example = 60)]
    pub interval: Option<u32>,
    /// Request timeout in seconds.
    #[schema(example = 10)]
    pub timeout: Option<u32>,
    /// Number of retries before marking as down. Defaults to `3`.
    #[schema(example = 3)]
    pub retries: Option<u32>,
    /// Whether the monitor is enabled.
    #[schema(example = true)]
    pub enabled: Option<bool>,
}

/// Request body for creating a new notification channel.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateNotificationChannelRequest {
    /// Human-readable name for the channel.
    #[schema(example = "Ops Slack")]
    pub name: String,
    /// Delivery configuration.
    pub config: NotificationChannelConfig,
}

/// Request body for updating a notification channel. All fields are optional.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateNotificationChannelRequest {
    /// Human-readable name for the channel.
    #[schema(example = "Ops Slack")]
    pub name: Option<String>,
    /// Delivery configuration. Replaces the entire config when provided.
    pub config: Option<NotificationChannelConfig>,
}

/// Request body for linking a notification channel to a monitor.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMonitorNotificationRequest {
    /// ID of the notification channel to link.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub channel_id: uuid::Uuid,
    /// Whether to also notify when the monitor recovers. Defaults to `true`.
    #[schema(example = true)]
    pub on_recovery: Option<bool>,
}

/// Request body for creating a new monitor.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateMonitorRequest {
    /// Human-readable name for the monitor.
    #[schema(example = "Production API")]
    pub name: String,
    /// Protocol-specific configuration.
    pub config: MonitorConfig,
    /// Polling interval in seconds. Minimum 10.
    #[schema(example = 60)]
    pub interval: u32,
    /// Request timeout in seconds.
    #[schema(example = 10)]
    pub timeout: u32,
    /// Number of retries before marking as down. Defaults to `3`.
    #[schema(example = 3)]
    pub retries: u32,
    /// Whether to start the monitor immediately. Defaults to `true`.
    #[schema(example = true)]
    pub enabled: Option<bool>,
}
