pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod monitor;
pub mod notify;
pub mod sweeper;

use dashmap::DashMap;
use jsonwebtoken::{DecodingKey, EncodingKey};
use openidconnect::Nonce;
use std::sync::Arc;
use std::time::Instant;
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
        NotificationChannelRepository, UserRepository,
    },
};
use rust_embed::RustEmbed;
use shared::models::{
    HttpBody, HttpMethod, Incident, Monitor, MonitorCheck, MonitorCheckStatus, MonitorConfig,
    MonitorNotification, NotificationChannel, NotificationChannelConfig, SseEvent,
};
use shared::{
    api::{
        CreateMonitorNotificationRequest, CreateMonitorRequest, CreateNotificationChannelRequest,
        UpdateMonitorRequest, UpdateNotificationChannelRequest,
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
    pub checks: Arc<dyn CheckRepository>,
    pub incidents: Arc<dyn IncidentRepository>,
    pub notification_channels: Arc<dyn NotificationChannelRepository>,
    pub monitor_notifications: Arc<dyn MonitorNotificationRepository>,
    pub users: Arc<dyn UserRepository>,
    pub engine: EngineHandle,
    pub stats: Arc<DashMap<Uuid, MonitorStats>>,
    pub event_tx: broadcast::Sender<SseEvent>,
    pub oidc_client: Arc<OidcClient>,
    pub pending_auth: Arc<DashMap<String, (Nonce, Instant)>>,
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
        crate::api::monitor_notifications::create_monitor_notification,
        crate::api::monitor_notifications::list_monitor_notifications,
        crate::api::monitor_notifications::delete_monitor_notification,
        crate::api::notification_channels::create_notification_channel,
        crate::api::notification_channels::list_notification_channels,
        crate::api::notification_channels::get_notification_channel,
        crate::api::notification_channels::update_notification_channel,
        crate::api::notification_channels::delete_notification_channel,
        crate::api::stats::stats_stream,
    ),
    components(
        schemas(
            CreateMonitorRequest,
            UpdateMonitorRequest,
            Monitor,
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
            ErrorBody
        )
    ),
    tags(
        (name = "monitors", description = "Monitor management"),
        (name = "notification-channels", description = "Notification channel management"),
        (name = "stats", description = "Real-time monitor statistics"),
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::require_auth,
        ));

    Router::new()
        .route("/auth/login", get(auth::handlers::login))
        .route("/auth/callback", get(auth::handlers::callback))
        .route("/auth/logout", post(auth::handlers::logout))
        .merge(protected)
        .merge(
            SwaggerUi::new("/docs")
                .url("/api/v1/openapi.json", ApiDoc::openapi())
                .config(SwaggerConfig::default().with_credentials(true)),
        )
        .with_state(state)
        .fallback(static_handler)
}
