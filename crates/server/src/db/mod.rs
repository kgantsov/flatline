pub mod sqlite_check;
pub mod sqlite_incident;
pub mod sqlite_monitor;
pub mod sqlite_monitor_notification;
pub mod sqlite_notification_channel;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mockall::automock;
use shared::api::{
    CreateMonitorCheckRequest, CreateMonitorNotificationRequest, CreateMonitorRequest,
    CreateNotificationChannelRequest, UpdateMonitorRequest, UpdateNotificationChannelRequest,
};
use shared::models::{Incident, Monitor, MonitorCheck, MonitorNotification, NotificationChannel};
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
    async fn list_for_monitor(
        &self,
        monitor_id: Uuid,
        limit: i64,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<MonitorCheck>, ApiError>;
}

#[automock]
#[async_trait]
pub trait NotificationChannelRepository: Send + Sync {
    async fn create(
        &self,
        input: CreateNotificationChannelRequest,
    ) -> Result<NotificationChannel, ApiError>;
    async fn list(&self) -> Result<Vec<NotificationChannel>, ApiError>;
    async fn get(&self, id: Uuid) -> Result<NotificationChannel, ApiError>;
    async fn update(
        &self,
        id: Uuid,
        input: UpdateNotificationChannelRequest,
    ) -> Result<NotificationChannel, ApiError>;
    async fn delete(&self, id: Uuid) -> Result<(), ApiError>;
}

#[automock]
#[async_trait]
pub trait MonitorNotificationRepository: Send + Sync {
    async fn create(
        &self,
        monitor_id: Uuid,
        input: CreateMonitorNotificationRequest,
    ) -> Result<MonitorNotification, ApiError>;
    async fn list_for_monitor(
        &self,
        monitor_id: Uuid,
    ) -> Result<Vec<MonitorNotification>, ApiError>;
    async fn delete(&self, monitor_id: Uuid, channel_id: Uuid) -> Result<(), ApiError>;
}

#[automock]
#[async_trait]
pub trait IncidentRepository: Send + Sync {
    async fn open(&self, monitor_id: Uuid, started_at: DateTime<Utc>)
    -> Result<Incident, ApiError>;
    async fn resolve(&self, id: Uuid, resolved_at: DateTime<Utc>) -> Result<Incident, ApiError>;
    async fn get_open_for_monitor(
        &self,
        monitor_id: Uuid,
    ) -> Result<Option<Incident>, ApiError>;
    async fn list_for_monitor(
        &self,
        monitor_id: Uuid,
        limit: i64,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<Incident>, ApiError>;
}
