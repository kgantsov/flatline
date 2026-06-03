pub mod api;
pub mod db;
pub mod error;

use std::sync::Arc;
use utoipa::OpenApi;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::db::MonitorRepository;
use crate::error::ErrorBody;
use shared::api::{CreateMonitorRequest, UpdateMonitorRequest};
use shared::models::{Monitor, MonitorConfig};

#[derive(Clone)]
pub struct AppState {
    pub monitors: Arc<dyn MonitorRepository>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::monitors::create_monitor,
        crate::api::monitors::get_monitors,
        crate::api::monitors::get_monitor,
        crate::api::monitors::delete_monitor,
        crate::api::monitors::update_monitor,
    ),
    components(schemas(CreateMonitorRequest, UpdateMonitorRequest, Monitor, MonitorConfig, ErrorBody)),
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
        .merge(SwaggerUi::new("/docs").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
