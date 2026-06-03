use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{Level, info};

use server::db::sqlite::SqliteMonitorRepository;
use server::{AppState, build_router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:flatline.db".to_string());

    let opts = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    info!("Database migrations applied");

    let state = AppState {
        monitors: Arc::new(SqliteMonitorRepository { pool }),
    };

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
