use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::Json};

use shared::api::{CreateMonitorRequest, UpdateMonitorRequest};
use shared::models::Monitor;
use uuid::Uuid;

use crate::AppState;
use crate::error::ApiError;

/// Create a new monitor.
#[utoipa::path(
    post,
    path = "/api/v1/monitors",
    request_body = CreateMonitorRequest,
    responses(
        (status = 201, description = "Monitor created successfully", body = Monitor),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn create_monitor(
    State(state): State<AppState>,
    Json(payload): Json<CreateMonitorRequest>,
) -> Result<(StatusCode, Json<Monitor>), ApiError> {
    let monitor = state.monitors.create(payload).await?;
    state.engine.start_monitor(state.clone(), monitor.clone()).await;
    Ok((StatusCode::CREATED, Json(monitor)))
}

/// Get a list of monitors
#[utoipa::path(
    get,
    path = "/api/v1/monitors",
    responses(
        (status = 200, description = "List of monitors retrieved successfully", body = [Monitor]),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn get_monitors(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<Monitor>>), ApiError> {
    let monitors = state.monitors.list().await?;
    Ok((StatusCode::OK, Json(monitors)))
}

/// Get a monitor by ID.
#[utoipa::path(
    get,
    path = "/api/v1/monitors/{monitor_id}",
    responses(
        (status = 200, description = "Monitor retrieved successfully", body = Monitor),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn get_monitor(
    State(state): State<AppState>,
    Path(monitor_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Monitor>), ApiError> {
    let monitor = state.monitors.get(monitor_id).await?;
    Ok((StatusCode::OK, Json(monitor)))
}

/// Update a monitor by ID.
#[utoipa::path(
    patch,
    path = "/api/v1/monitors/{monitor_id}",
    request_body = UpdateMonitorRequest,
    responses(
        (status = 200, description = "Monitor updated successfully", body = Monitor),
        (status = 404, description = "Monitor not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn update_monitor(
    State(state): State<AppState>,
    Path(monitor_id): Path<Uuid>,
    Json(payload): Json<UpdateMonitorRequest>,
) -> Result<(StatusCode, Json<Monitor>), ApiError> {
    let monitor = state.monitors.update(monitor_id, payload).await?;
    state.engine.restart_monitor(state.clone(), monitor.clone()).await;
    Ok((StatusCode::OK, Json(monitor)))
}

/// Delete a monitor by ID.
#[utoipa::path(
    delete,
    path = "/api/v1/monitors/{monitor_id}",
    responses(
        (status = 204, description = "Monitor retrieved successfully", body = Monitor),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn delete_monitor(
    State(state): State<AppState>,
    Path(monitor_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.engine.stop_monitor(monitor_id).await;
    state.monitors.delete(monitor_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
