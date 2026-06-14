use dashmap::DashMap;
use server::db::sqlite_incident::SqliteIncidentRepository;
use server::db::sqlite_monitor_notification::SqliteMonitorNotificationRepository;
use server::db::sqlite_notification_channel::SqliteNotificationChannelRepository;
use server::monitor::engine::{EngineHandle, MonitorEngine};
use shared::models::MonitorStats;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{Level, info};
use uuid::Uuid;

use server::db::sqlite_check::SqliteCheckRepository;
use server::db::sqlite_monitor::SqliteMonitorRepository;
use server::{AppState, build_router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:flatline.db".to_string());

    let opts = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .pragma("foreign_keys", "ON");
    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    info!("Database migrations applied");

    let engine_handle = EngineHandle::new();
    let state = AppState {
        monitors: Arc::new(SqliteMonitorRepository { pool: pool.clone() }),
        checks: Arc::new(SqliteCheckRepository { pool: pool.clone() }),
        incidents: Arc::new(SqliteIncidentRepository { pool: pool.clone() }),
        notification_channels: Arc::new(SqliteNotificationChannelRepository { pool: pool.clone() }),
        monitor_notifications: Arc::new(SqliteMonitorNotificationRepository { pool: pool.clone() }),
        engine: engine_handle.clone(),
        stats: Arc::new(DashMap::<Uuid, MonitorStats>::new()),
    };

    let mut engine = MonitorEngine::new(state.clone(), engine_handle);
    engine.start().await?;

    let app = build_router(state);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server running on http://{}", addr);
    info!("Swagger UI available at http://{}/docs", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
