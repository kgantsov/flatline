pub mod api;
pub mod db;
pub mod error;
pub mod monitor;
pub mod notify;

use std::sync::Arc;
use utoipa::OpenApi;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::db::{
    CheckRepository, IncidentRepository, MonitorNotificationRepository, MonitorRepository,
    NotificationChannelRepository,
};
use crate::error::ErrorBody;
use crate::monitor::engine::EngineHandle;
use shared::api::{
    CreateMonitorNotificationRequest, CreateMonitorRequest, CreateNotificationChannelRequest,
    UpdateMonitorRequest, UpdateNotificationChannelRequest,
};
use shared::models::{
    HttpMethod, Incident, Monitor, MonitorCheck, MonitorCheckStatus, MonitorConfig,
    MonitorNotification, NotificationChannel, NotificationChannelConfig,
};

#[derive(Clone)]
pub struct AppState {
    pub monitors: Arc<dyn MonitorRepository>,
    pub checks: Arc<dyn CheckRepository>,
    pub incidents: Arc<dyn IncidentRepository>,
    pub notification_channels: Arc<dyn NotificationChannelRepository>,
    pub monitor_notifications: Arc<dyn MonitorNotificationRepository>,
    pub engine: EngineHandle,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::monitors::create_monitor,
        crate::api::monitors::get_monitors,
        crate::api::monitors::get_monitor,
        crate::api::monitors::delete_monitor,
        crate::api::monitors::update_monitor,
        crate::api::monitors::get_monitor_checks,
        crate::api::monitors::get_monitor_incidents,
        crate::api::monitor_notifications::create_monitor_notification,
        crate::api::monitor_notifications::list_monitor_notifications,
        crate::api::monitor_notifications::delete_monitor_notification,
        crate::api::notification_channels::create_notification_channel,
        crate::api::notification_channels::list_notification_channels,
        crate::api::notification_channels::get_notification_channel,
        crate::api::notification_channels::update_notification_channel,
        crate::api::notification_channels::delete_notification_channel,
    ),
    components(
        schemas(
            CreateMonitorRequest,
            UpdateMonitorRequest,
            Monitor,
            MonitorConfig,
            MonitorCheck,
            HttpMethod,
            MonitorCheckStatus,
            Incident,
            CreateNotificationChannelRequest,
            UpdateNotificationChannelRequest,
            NotificationChannel,
            NotificationChannelConfig,
            CreateMonitorNotificationRequest,
            MonitorNotification,
            ErrorBody
        )
    ),
    tags(
        (name = "monitors", description = "Monitor management"),
        (name = "notification-channels", description = "Notification channel management"),
    ),
    info(title = "Flatline API", version = "0.1.0", description = "Open-source uptime monitor API"),
)]
pub struct ApiDoc;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/monitors", post(api::monitors::create_monitor))
        .route("/api/v1/monitors", get(api::monitors::get_monitors))
        .route(
            "/api/v1/monitors/:monitor_id",
            get(api::monitors::get_monitor),
        )
        .route(
            "/api/v1/monitors/:monitor_id",
            delete(api::monitors::delete_monitor),
        )
        .route(
            "/api/v1/monitors/:monitor_id",
            patch(api::monitors::update_monitor),
        )
        .route(
            "/api/v1/monitors/:monitor_id/checks",
            get(api::monitors::get_monitor_checks),
        )
        .route(
            "/api/v1/monitors/:monitor_id/incidents",
            get(api::monitors::get_monitor_incidents),
        )
        .route(
            "/api/v1/monitors/:monitor_id/notifications",
            post(api::monitor_notifications::create_monitor_notification),
        )
        .route(
            "/api/v1/monitors/:monitor_id/notifications",
            get(api::monitor_notifications::list_monitor_notifications),
        )
        .route(
            "/api/v1/monitors/:monitor_id/notifications/:channel_id",
            delete(api::monitor_notifications::delete_monitor_notification),
        )
        .route(
            "/api/v1/notification-channels",
            post(api::notification_channels::create_notification_channel),
        )
        .route(
            "/api/v1/notification-channels",
            get(api::notification_channels::list_notification_channels),
        )
        .route(
            "/api/v1/notification-channels/:channel_id",
            get(api::notification_channels::get_notification_channel),
        )
        .route(
            "/api/v1/notification-channels/:channel_id",
            patch(api::notification_channels::update_notification_channel),
        )
        .route(
            "/api/v1/notification-channels/:channel_id",
            delete(api::notification_channels::delete_notification_channel),
        )
        .merge(SwaggerUi::new("/docs").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
