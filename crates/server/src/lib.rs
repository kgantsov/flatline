pub mod api;
pub mod db;
pub mod error;
pub mod monitor;

use std::sync::Arc;
use utoipa::OpenApi;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::db::{CheckRepository, IncidentRepository, MonitorRepository};
use crate::error::ErrorBody;
use crate::monitor::engine::EngineHandle;
use shared::models::{HttpMethod, Incident, Monitor, MonitorCheckStatus, MonitorConfig};
use shared::{
    api::{CreateMonitorRequest, UpdateMonitorRequest},
    models::MonitorCheck,
};

#[derive(Clone)]
pub struct AppState {
    pub monitors: Arc<dyn MonitorRepository>,
    pub checks: Arc<dyn CheckRepository>,
    pub incidents: Arc<dyn IncidentRepository>,
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
            ErrorBody
        )
    ),
    tags((name = "monitors", description = "Monitor management")),
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
        .merge(SwaggerUi::new("/docs").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
