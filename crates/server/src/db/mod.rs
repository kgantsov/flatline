pub mod sqlite;

use async_trait::async_trait;
use mockall::automock;
use shared::api::{CreateMonitorRequest, UpdateMonitorRequest};
use shared::models::Monitor;
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
