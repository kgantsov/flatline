use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::MonitorConfig;

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
    /// Whether the monitor is enabled.
    #[schema(example = true)]
    pub enabled: Option<bool>,
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
    /// Whether to start the monitor immediately. Defaults to `true`.
    #[schema(example = true)]
    pub enabled: Option<bool>,
}
