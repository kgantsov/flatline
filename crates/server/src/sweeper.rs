use std::time::Duration;

use crate::AppState;

pub async fn run_sweeper(state: AppState) {
    let interval = Duration::from_secs(state.config.sweep_interval_seconds);
    let mut ticker = tokio::time::interval(interval);

    loop {
        ticker.tick().await;
        if let Err(e) = sweep_once(&state).await {
            tracing::error!("Error during monitor checks sweep: {:?}", e);
        }
    }
}

async fn sweep_once(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Sweep old checks...");
    state
        .checks
        .delete_old_checks(
            chrono::Utc::now()
                - chrono::Duration::days(
                    state
                        .config
                        .monitor_checks_retention_days
                        .try_into()
                        .unwrap_or(90),
                ),
        )
        .await?;

    tracing::info!("Sweep old checks done.");
    Ok(())
}
