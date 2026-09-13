pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod monitor;
pub mod notify;
pub mod request_id;
pub mod sweeper;

use dashmap::DashMap;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;
use tokio::sync::broadcast;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};
use uuid::Uuid;

use axum::{
    Router,
    http::{StatusCode, Uri, header},
    middleware,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use utoipa_swagger_ui::{Config as SwaggerConfig, SwaggerUi};

use crate::auth::oidc::OidcClient;
use crate::error::ErrorBody;
use crate::monitor::engine::EngineHandle;
use crate::{
    config::Config,
    db::{
        CheckRepository, IncidentRepository, MonitorNotificationRepository, MonitorRepository,
        NotificationChannelRepository, StatusPageMonitorRepository, StatusPageRepository,
        UserRepository,
    },
};
use rust_embed::RustEmbed;
use shared::models::{
    HttpBody, HttpMethod, Incident, Monitor, MonitorCheck, MonitorCheckStatus, MonitorConfig,
    MonitorNotification, MonitorSummary, NotificationChannel, NotificationChannelConfig,
    PublicStatusPage, SseEvent, StatusPage, StatusPageMonitor,
};
use shared::{
    api::{
        AddStatusPageMonitorRequest, CreateMonitorNotificationRequest, CreateMonitorRequest,
        CreateNotificationChannelRequest, CreateStatusPageRequest, UpdateMonitorRequest,
        UpdateNotificationChannelRequest, UpdateStatusPageRequest,
    },
    models::MonitorStats,
};

#[derive(RustEmbed)]
#[folder = "../../dist"]
struct Assets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Assets::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if let Some(index) = Assets::get("index.html") {
                return ([(header::CONTENT_TYPE, "text/html")], index.data).into_response();
            }
            (StatusCode::NOT_FOUND, "404 Not Found").into_response()
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub monitors: Arc<dyn MonitorRepository>,
    pub status_pages: Arc<dyn StatusPageRepository>,
    pub status_page_monitors: Arc<dyn StatusPageMonitorRepository>,
    pub checks: Arc<dyn CheckRepository>,
    pub incidents: Arc<dyn IncidentRepository>,
    pub notification_channels: Arc<dyn NotificationChannelRepository>,
    pub monitor_notifications: Arc<dyn MonitorNotificationRepository>,
    pub users: Arc<dyn UserRepository>,
    pub engine: EngineHandle,
    pub stats: Arc<DashMap<Uuid, MonitorStats>>,
    pub event_tx: broadcast::Sender<SseEvent>,
    pub oidc_client: Arc<OidcClient>,
    pub http_client: reqwest::Client,
    pub jwt_encoding_key: Arc<EncodingKey>,
    pub jwt_decoding_key: Arc<DecodingKey>,
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
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
        crate::api::monitors::get_monitor_incident_history,
        crate::api::monitor_notifications::create_monitor_notification,
        crate::api::monitor_notifications::list_monitor_notifications,
        crate::api::monitor_notifications::delete_monitor_notification,
        crate::api::notification_channels::create_notification_channel,
        crate::api::notification_channels::list_notification_channels,
        crate::api::notification_channels::get_notification_channel,
        crate::api::notification_channels::update_notification_channel,
        crate::api::notification_channels::delete_notification_channel,
        crate::api::stats::stats_stream,
        crate::api::status_pages::create_status_page,
        crate::api::status_pages::list_status_pages,
        crate::api::status_pages::get_status_page,
        crate::api::status_pages::update_status_page,
        crate::api::status_pages::delete_status_page,
        crate::api::status_pages::add_monitor_to_page,
        crate::api::status_pages::list_page_monitors,
        crate::api::status_pages::remove_monitor_from_page,
        crate::api::status_pages::get_public_status_page,
    ),
    components(
        schemas(
            CreateMonitorRequest,
            UpdateMonitorRequest,
            Monitor,
            MonitorSummary,
            MonitorConfig,
            HttpBody,
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
            ErrorBody,
            CreateStatusPageRequest,
            UpdateStatusPageRequest,
            AddStatusPageMonitorRequest,
            StatusPage,
            StatusPageMonitor,
            PublicStatusPage,
        )
    ),
    tags(
        (name = "monitors", description = "Monitor management"),
        (name = "notification-channels", description = "Notification channel management"),
        (name = "stats", description = "Real-time monitor statistics"),
        (name = "status-pages", description = "Status page management"),
    ),
    modifiers(&SecurityAddon),
    security(("bearerAuth" = [])),
    info(title = "Flatline API", version = "0.1.0", description = "Open-source uptime monitor API"),
)]
pub struct ApiDoc;

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/auth/me", get(auth::handlers::me))
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
            "/api/v1/monitors/:monitor_id/incident-history",
            get(api::monitors::get_monitor_incident_history),
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
        .route("/api/v1/stats/stream", get(api::stats::stats_stream))
        .route(
            "/api/v1/status-pages",
            post(api::status_pages::create_status_page),
        )
        .route(
            "/api/v1/status-pages",
            get(api::status_pages::list_status_pages),
        )
        .route(
            "/api/v1/status-pages/:page_id",
            get(api::status_pages::get_status_page),
        )
        .route(
            "/api/v1/status-pages/:page_id",
            patch(api::status_pages::update_status_page),
        )
        .route(
            "/api/v1/status-pages/:page_id",
            delete(api::status_pages::delete_status_page),
        )
        .route(
            "/api/v1/status-pages/:page_id/monitors",
            post(api::status_pages::add_monitor_to_page),
        )
        .route(
            "/api/v1/status-pages/:page_id/monitors",
            get(api::status_pages::list_page_monitors),
        )
        .route(
            "/api/v1/status-pages/:page_id/monitors/:monitor_id",
            delete(api::status_pages::remove_monitor_from_page),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ));

    Router::new()
        .route("/auth/login", get(auth::handlers::login))
        .route("/auth/callback", get(auth::handlers::callback))
        .route("/auth/logout", post(auth::handlers::logout))
        .route("/status/:slug", get(api::status_pages::get_public_status_page))
        .merge(protected)
        .merge(
            SwaggerUi::new("/docs")
                .url("/api/v1/openapi.json", ApiDoc::openapi())
                .config(SwaggerConfig::default().with_credentials(true)),
        )
        .with_state(state)
        .fallback(static_handler)
        .layer(middleware::from_fn(request_id::request_id))
}
