use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::Json};
use uuid::Uuid;

use shared::api::CreateMonitorNotificationRequest;
use shared::models::MonitorNotification;

use crate::AppState;
use crate::error::ApiError;

/// Add a notification channel to a monitor.
#[utoipa::path(
    post,
    path = "/api/v1/monitors/{monitor_id}/notifications",
    request_body = CreateMonitorNotificationRequest,
    responses(
        (status = 201, description = "Notification link created successfully", body = MonitorNotification),
        (status = 400, description = "Channel already linked or invalid request", body = ErrorBody),
        (status = 404, description = "Monitor or channel not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn create_monitor_notification(
    State(state): State<AppState>,
    Path(monitor_id): Path<Uuid>,
    Json(payload): Json<CreateMonitorNotificationRequest>,
) -> Result<(StatusCode, Json<MonitorNotification>), ApiError> {
    // Verify the monitor exists.
    state.monitors.get(monitor_id).await?;
    // Verify the channel exists.
    state
        .notification_channels
        .get(payload.channel_id)
        .await?;

    let notification = state
        .monitor_notifications
        .create(monitor_id, payload)
        .await?;
    Ok((StatusCode::CREATED, Json(notification)))
}

/// List notification channels linked to a monitor.
#[utoipa::path(
    get,
    path = "/api/v1/monitors/{monitor_id}/notifications",
    responses(
        (status = 200, description = "List of monitor notification links", body = [MonitorNotification]),
        (status = 404, description = "Monitor not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn list_monitor_notifications(
    State(state): State<AppState>,
    Path(monitor_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<MonitorNotification>>), ApiError> {
    // Verify the monitor exists.
    state.monitors.get(monitor_id).await?;
    let notifications = state
        .monitor_notifications
        .list_for_monitor(monitor_id)
        .await?;
    Ok((StatusCode::OK, Json(notifications)))
}

/// Remove a notification channel from a monitor.
#[utoipa::path(
    delete,
    path = "/api/v1/monitors/{monitor_id}/notifications/{channel_id}",
    responses(
        (status = 204, description = "Notification link removed successfully"),
        (status = 404, description = "Link not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn delete_monitor_notification(
    State(state): State<AppState>,
    Path((monitor_id, channel_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .monitor_notifications
        .delete(monitor_id, channel_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
