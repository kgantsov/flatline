use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use uuid::Uuid;

/// Slug must be 1–100 chars, lowercase alphanumeric, hyphens, or underscores,
/// and must start with an alphanumeric character.
fn validate_slug(slug: &str) -> Result<(), ApiError> {
    if slug.is_empty() || slug.len() > 100 {
        return Err(ApiError::BadRequest(
            "Slug must be between 1 and 100 characters".into(),
        ));
    }
    let valid = slug
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric())
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !valid {
        return Err(ApiError::BadRequest(
            "Slug may only contain lowercase letters, digits, hyphens, and underscores, and must start with a letter or digit".into(),
        ));
    }
    Ok(())
}

use shared::api::{AddStatusPageMonitorRequest, CreateStatusPageRequest, UpdateStatusPageRequest};
use shared::models::{
    PublicMonitor, PublicMonitorStatus, PublicStatusPage, StatusPage, StatusPageMonitor,
};

use crate::AppState;
use crate::error::ApiError;

/// Create a new status page.
#[utoipa::path(
    post,
    path = "/api/v1/status-pages",
    request_body = CreateStatusPageRequest,
    responses(
        (status = 201, description = "Status page created successfully", body = StatusPage),
        (status = 400, description = "Invalid request body", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "status-pages"
)]
pub async fn create_status_page(
    State(state): State<AppState>,
    Json(payload): Json<CreateStatusPageRequest>,
) -> Result<(StatusCode, Json<StatusPage>), ApiError> {
    validate_slug(&payload.slug)?;
    let page = state.status_pages.create(payload).await?;
    Ok((StatusCode::CREATED, Json(page)))
}

/// List all status pages.
#[utoipa::path(
    get,
    path = "/api/v1/status-pages",
    responses(
        (status = 200, description = "List of status pages", body = [StatusPage]),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "status-pages"
)]
pub async fn list_status_pages(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<StatusPage>>), ApiError> {
    let pages = state.status_pages.list().await?;
    Ok((StatusCode::OK, Json(pages)))
}

/// Get a status page by ID.
#[utoipa::path(
    get,
    path = "/api/v1/status-pages/{page_id}",
    responses(
        (status = 200, description = "Status page retrieved successfully", body = StatusPage),
        (status = 404, description = "Status page not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "status-pages"
)]
pub async fn get_status_page(
    State(state): State<AppState>,
    Path(page_id): Path<Uuid>,
) -> Result<(StatusCode, Json<StatusPage>), ApiError> {
    let page = state.status_pages.get(page_id).await?;
    Ok((StatusCode::OK, Json(page)))
}

/// Update a status page by ID.
#[utoipa::path(
    patch,
    path = "/api/v1/status-pages/{page_id}",
    request_body = UpdateStatusPageRequest,
    responses(
        (status = 200, description = "Status page updated successfully", body = StatusPage),
        (status = 404, description = "Status page not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "status-pages"
)]
pub async fn update_status_page(
    State(state): State<AppState>,
    Path(page_id): Path<Uuid>,
    Json(payload): Json<UpdateStatusPageRequest>,
) -> Result<(StatusCode, Json<StatusPage>), ApiError> {
    if let Some(slug) = &payload.slug {
        validate_slug(slug)?;
    }
    let page = state.status_pages.update(page_id, payload).await?;
    Ok((StatusCode::OK, Json(page)))
}

/// Delete a status page by ID.
#[utoipa::path(
    delete,
    path = "/api/v1/status-pages/{page_id}",
    responses(
        (status = 204, description = "Status page deleted successfully"),
        (status = 404, description = "Status page not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "status-pages"
)]
pub async fn delete_status_page(
    State(state): State<AppState>,
    Path(page_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.status_pages.delete(page_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Add a monitor to a status page.
#[utoipa::path(
    post,
    path = "/api/v1/status-pages/{page_id}/monitors",
    request_body = AddStatusPageMonitorRequest,
    responses(
        (status = 201, description = "Monitor added to status page", body = StatusPageMonitor),
        (status = 400, description = "Monitor already on page", body = ErrorBody),
        (status = 404, description = "Status page not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "status-pages"
)]
pub async fn add_monitor_to_page(
    State(state): State<AppState>,
    Path(page_id): Path<Uuid>,
    Json(payload): Json<AddStatusPageMonitorRequest>,
) -> Result<(StatusCode, Json<StatusPageMonitor>), ApiError> {
    // Verify the page exists
    state.status_pages.get(page_id).await?;
    let link = state.status_page_monitors.add(page_id, payload).await?;
    Ok((StatusCode::CREATED, Json(link)))
}

/// List monitors on a status page.
#[utoipa::path(
    get,
    path = "/api/v1/status-pages/{page_id}/monitors",
    responses(
        (status = 200, description = "Monitors on this status page", body = [StatusPageMonitor]),
        (status = 404, description = "Status page not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "status-pages"
)]
pub async fn list_page_monitors(
    State(state): State<AppState>,
    Path(page_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<StatusPageMonitor>>), ApiError> {
    state.status_pages.get(page_id).await?;
    let links = state.status_page_monitors.list_for_page(page_id).await?;
    Ok((StatusCode::OK, Json(links)))
}

/// Remove a monitor from a status page.
#[utoipa::path(
    delete,
    path = "/api/v1/status-pages/{page_id}/monitors/{monitor_id}",
    responses(
        (status = 204, description = "Monitor removed from status page"),
        (status = 404, description = "Not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "status-pages"
)]
pub async fn remove_monitor_from_page(
    State(state): State<AppState>,
    Path((page_id, monitor_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .status_page_monitors
        .remove(page_id, monitor_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get the public status page by slug (no authentication required).
#[utoipa::path(
    get,
    path = "/status/{slug}",
    responses(
        (status = 200, description = "Public status page data", body = PublicStatusPage),
        (status = 404, description = "Status page not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    ),
    tag = "status-pages"
)]
pub async fn get_public_status_page(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<(StatusCode, Json<PublicStatusPage>), ApiError> {
    let page = state.status_pages.get_by_slug(&slug).await?;
    let links = state.status_page_monitors.list_for_page(page.id).await?;

    let now = Utc::now();
    let mut monitors = Vec::with_capacity(links.len());
    for link in links {
        let monitor = state.monitors.get(link.monitor_id).await?;
        let stats = state.stats.get(&monitor.id).map(|s| s.clone());
        // Fetch enough incidents to cover 90 days; incidents are ordered newest-first.
        let incidents = state
            .incidents
            .list_for_monitor(monitor.id, 200, None)
            .await
            .unwrap_or_default();
        // Build a 90-element vec: index 0 = 89 days ago, index 89 = today.
        // Each value is the total downtime minutes for that calendar day.
        let day_downtime_minutes: Vec<u32> = (0..90_i64)
            .rev()
            .map(|days_ago| {
                let day = (now - chrono::Duration::days(days_ago)).date_naive();
                let day_start = day.and_hms_opt(0, 0, 0).unwrap().and_utc();
                let day_end = day
                    .succ_opt()
                    .unwrap_or(day)
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                incidents
                    .iter()
                    .filter_map(|inc| {
                        let inc_end = inc.resolved_at.unwrap_or(now);
                        // Clamp the incident window to this calendar day.
                        let overlap_start = inc.started_at.max(day_start);
                        let overlap_end = inc_end.min(day_end);
                        if overlap_end > overlap_start {
                            let secs = (overlap_end - overlap_start).num_seconds().max(0) as u32;
                            Some(secs / 60)
                        } else {
                            None
                        }
                    })
                    .sum()
            })
            .collect();
        let is_up = incidents.iter().all(|inc| inc.resolved_at.is_some());
        let public_monitor = PublicMonitor {
            id: monitor.id,
            name: monitor.name,
        };
        monitors.push(PublicMonitorStatus {
            monitor: public_monitor,
            is_up,
            stats,
            day_downtime_minutes,
        });
    }

    Ok((StatusCode::OK, Json(PublicStatusPage { page, monitors })))
}
