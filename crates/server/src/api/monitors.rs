use axum::extract::{Path, Query};
use axum::{extract::State, http::StatusCode, response::Json};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use shared::api::{CreateMonitorRequest, UpdateMonitorRequest};
use shared::models::{Incident, Monitor, MonitorCheck};
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
    state
        .engine
        .start_monitor(state.clone(), monitor.clone())
        .await;
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
    state
        .engine
        .restart_monitor(state.clone(), monitor.clone())
        .await;
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

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ChecksQuery {
    /// Maximum number of checks to return (default: 25, max: 100).
    #[serde(default = "default_checks_limit")]
    pub limit: i64,
    /// Return only checks recorded before this timestamp (RFC3339).
    pub before: Option<DateTime<Utc>>,
}

fn default_checks_limit() -> i64 {
    25
}

/// Get a list of checks for a monitor by ID.
#[utoipa::path(
    get,
    path = "/api/v1/monitors/{monitor_id}/checks",
    params(ChecksQuery),
    responses(
        (status = 200, description = "List of monitor checks", body = [MonitorCheck]),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn get_monitor_checks(
    State(state): State<AppState>,
    Path(monitor_id): Path<Uuid>,
    Query(query): Query<ChecksQuery>,
) -> Result<(StatusCode, Json<Vec<MonitorCheck>>), ApiError> {
    let limit = query.limit.min(100);
    let monitor_checks = state
        .checks
        .list_for_monitor(monitor_id, limit, query.before)
        .await?;
    Ok((StatusCode::OK, Json(monitor_checks)))
}

/// Get a list of incdents for a monitor by ID.
#[utoipa::path(
    get,
    path = "/api/v1/monitors/{monitor_id}/incidents",
    params(IncidentsQuery),
    responses(
        (status = 200, description = "List of monitor check", body = [Incident]),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "monitors"
)]
pub async fn get_monitor_incidents(
    State(state): State<AppState>,
    Path(monitor_id): Path<Uuid>,
    Query(query): Query<IncidentsQuery>,
) -> Result<(StatusCode, Json<Vec<Incident>>), ApiError> {
    let limit = query.limit.min(100);
    let incidents = state
        .incidents
        .list_for_monitor(monitor_id, limit, query.before)
        .await?;
    Ok((StatusCode::OK, Json(incidents)))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct IncidentsQuery {
    /// Maximum number of incidents to return (default: 25, max: 100).
    #[serde(default = "default_incidents_limit")]
    pub limit: i64,
    /// Return only incidents started before this timestamp (RFC3339).
    pub before: Option<DateTime<Utc>>,
}

fn default_incidents_limit() -> i64 {
    25
}
