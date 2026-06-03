pub mod sqlite_check;
pub mod sqlite_monitor;

use async_trait::async_trait;
use mockall::automock;
use shared::api::{CreateMonitorCheckRequest, CreateMonitorRequest, UpdateMonitorRequest};
use shared::models::{Monitor, MonitorCheck};
use uuid::Uuid;

use crate::error::ApiError;

#[automock]
#[async_trait]
pub trait MonitorRepository: Send + Sync {
    async fn create(&self, input: CreateMonitorRequest) -> Result<Monitor, ApiError>;
    async fn list(&self) -> Result<Vec<Monitor>, ApiError>;
    async fn get(&self, id: Uuid) -> Result<Monitor, ApiError>;
    async fn update(&self, id: Uuid, input: UpdateMonitorRequest) -> Result<Monitor, ApiError>;
    async fn delete(&self, id: Uuid) -> Result<(), ApiError>;
}

#[automock]
#[async_trait]
pub trait CheckRepository: Send + Sync {
    async fn create(&self, check: CreateMonitorCheckRequest) -> Result<MonitorCheck, ApiError>;
    async fn list_for_monitor(&self, monitor_id: Uuid, limit: i64) -> Result<Vec<MonitorCheck>, ApiError>;
}
