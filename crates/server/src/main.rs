use clap::Parser;
use dashmap::DashMap;
use jsonwebtoken::{DecodingKey, EncodingKey};
use server::auth::oidc::build_oidc_client;
use server::config::init_config;
use server::db::sqlite_check::SqliteCheckRepository;
use server::db::sqlite_incident::SqliteIncidentRepository;
use server::db::sqlite_monitor::SqliteMonitorRepository;
use server::db::sqlite_monitor_notification::SqliteMonitorNotificationRepository;
use server::db::sqlite_notification_channel::SqliteNotificationChannelRepository;
use server::db::sqlite_user::SqliteUserRepository;
use server::monitor::engine::{EngineHandle, MonitorEngine};
use server::sweeper::run_sweeper;
use server::{AppState, build_router};
use shared::models::MonitorStats;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{Level, info};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    author = "Kostiantyn Hantsov",
    version,
    about = "Flatline - A simple uptime monitoring service"
)]
struct Args {
    #[arg(
        short,
        long,
        value_name = "ADDRESS",
        help = "Address to listen on",
        default_value = "0.0.0.0:3000"
    )]
    address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let args = Args::parse();

    let config = init_config();

    let opts = SqliteConnectOptions::from_str(&config.database_url)?
        .create_if_missing(true)
        .pragma("foreign_keys", "ON");
    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    info!("Database migrations applied");

    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let oidc_client = Arc::new(
        build_oidc_client(
            &config.oauth_issuer_url,
            &config.oauth_client_id,
            &config.oauth_client_secret,
            &config.oauth_redirect_url,
        )
        .await?,
    );

    let jwt_encoding_key = Arc::new(EncodingKey::from_secret(config.jwt_secret.as_bytes()));
    let jwt_decoding_key = Arc::new(DecodingKey::from_secret(config.jwt_secret.as_bytes()));

    let (event_tx, _) = broadcast::channel(256);
    let engine_handle = EngineHandle::new();
    let state = AppState {
        config,
        monitors: Arc::new(SqliteMonitorRepository { pool: pool.clone() }),
        checks: Arc::new(SqliteCheckRepository { pool: pool.clone() }),
        incidents: Arc::new(SqliteIncidentRepository { pool: pool.clone() }),
        notification_channels: Arc::new(SqliteNotificationChannelRepository { pool: pool.clone() }),
        monitor_notifications: Arc::new(SqliteMonitorNotificationRepository { pool: pool.clone() }),
        users: Arc::new(SqliteUserRepository { pool: pool.clone() }),
        engine: engine_handle.clone(),
        stats: Arc::new(DashMap::<Uuid, MonitorStats>::new()),
        event_tx,
        oidc_client,
        pending_auth: Arc::new(DashMap::new()),
        http_client,
        jwt_encoding_key,
        jwt_decoding_key,
    };

    let mut engine = MonitorEngine::new(state.clone(), engine_handle);
    engine.start().await?;

    tokio::spawn(run_sweeper(state.clone()));

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(&args.address).await?;
    info!("Server running on http://{}", args.address);
    info!("Swagger UI available at http://{}/docs", args.address);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
