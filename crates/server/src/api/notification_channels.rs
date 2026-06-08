use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::Json};
use uuid::Uuid;

use shared::api::{CreateNotificationChannelRequest, UpdateNotificationChannelRequest};
use shared::models::NotificationChannel;

use crate::AppState;
use crate::error::ApiError;

/// Create a new notification channel.
#[utoipa::path(
    post,
    path = "/api/v1/notification-channels",
    request_body = CreateNotificationChannelRequest,
    responses(
        (status = 201, description = "Notification channel created successfully", body = NotificationChannel),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "notification-channels"
)]
pub async fn create_notification_channel(
    State(state): State<AppState>,
    Json(payload): Json<CreateNotificationChannelRequest>,
) -> Result<(StatusCode, Json<NotificationChannel>), ApiError> {
    let channel = state.notification_channels.create(payload).await?;
    Ok((StatusCode::CREATED, Json(channel)))
}

/// List all notification channels.
#[utoipa::path(
    get,
    path = "/api/v1/notification-channels",
    responses(
        (status = 200, description = "List of notification channels", body = [NotificationChannel]),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "notification-channels"
)]
pub async fn list_notification_channels(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<NotificationChannel>>), ApiError> {
    let channels = state.notification_channels.list().await?;
    Ok((StatusCode::OK, Json(channels)))
}

/// Get a notification channel by ID.
#[utoipa::path(
    get,
    path = "/api/v1/notification-channels/{channel_id}",
    responses(
        (status = 200, description = "Notification channel retrieved successfully", body = NotificationChannel),
        (status = 404, description = "Notification channel not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "notification-channels"
)]
pub async fn get_notification_channel(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
) -> Result<(StatusCode, Json<NotificationChannel>), ApiError> {
    let channel = state.notification_channels.get(channel_id).await?;
    Ok((StatusCode::OK, Json(channel)))
}

/// Update a notification channel by ID.
#[utoipa::path(
    patch,
    path = "/api/v1/notification-channels/{channel_id}",
    request_body = UpdateNotificationChannelRequest,
    responses(
        (status = 200, description = "Notification channel updated successfully", body = NotificationChannel),
        (status = 404, description = "Notification channel not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "notification-channels"
)]
pub async fn update_notification_channel(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
    Json(payload): Json<UpdateNotificationChannelRequest>,
) -> Result<(StatusCode, Json<NotificationChannel>), ApiError> {
    let channel = state
        .notification_channels
        .update(channel_id, payload)
        .await?;
    Ok((StatusCode::OK, Json(channel)))
}

/// Delete a notification channel by ID.
#[utoipa::path(
    delete,
    path = "/api/v1/notification-channels/{channel_id}",
    responses(
        (status = 204, description = "Notification channel deleted successfully"),
        (status = 404, description = "Notification channel not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "notification-channels"
)]
pub async fn delete_notification_channel(
    State(state): State<AppState>,
    Path(channel_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.notification_channels.delete(channel_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
