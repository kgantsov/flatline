use std::sync::{Arc, Mutex};

use shared::{
    api::CreateMonitorCheckRequest,
    models::{Monitor, MonitorConfig},
};
use tokio::time::{self, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use crate::{
    AppState,
    monitor::{
        checker::{Checker, Status},
        http::HttpChecker,
    },
};

pub struct MonitorWorker {
    state: AppState,
    monitor: Monitor,
    cancellation_token: CancellationToken,
    status: Arc<Mutex<Status>>,
}

impl MonitorWorker {
    pub fn new(state: AppState, monitor: Monitor) -> Self {
        Self {
            state,
            monitor,
            cancellation_token: CancellationToken::new(),
            status: Arc::new(Mutex::new(Status::Unknown)),
        }
    }

    fn build_checker(monitor: &Monitor) -> Box<dyn Checker> {
        match monitor.config {
            MonitorConfig::Http {
                ref url,
                ref method,
                ref expected_status,
            } => Box::new(HttpChecker::new(
                url.clone(),
                method.clone(),
                expected_status.clone(),
            )),
        }
    }

    pub async fn start(&self) {
        debug!(
            "Starting monitor worker for monitor: {} {}",
            self.monitor.id, self.monitor.name
        );

        let monitor_id = self.monitor.id;
        self.state
            .checks
            .list_for_monitor(monitor_id, 1)
            .await
            .ok()
            .and_then(|mut checks| checks.pop())
            .map(|last_check| {
                let mut guard = self.status.lock().unwrap_or_else(|e| e.into_inner());
                *guard = match last_check.status {
                    shared::models::MonitorCheckStatus::Up => Status::Up,
                    shared::models::MonitorCheckStatus::Down => Status::Down,
                };
            });

        let token = self.cancellation_token.clone();
        let monitor = self.monitor.clone();

        let interval_secs = Duration::from_secs(self.monitor.interval as u64);
        let mut interval = time::interval_at(time::Instant::now() + interval_secs, interval_secs);

        let status = Arc::clone(&self.status);
        let state = self.state.clone();

        tokio::spawn(async move {
            let checker = Self::build_checker(&monitor);
            loop {
                tokio::select! {
                    _ = token.cancelled() => break,
                    _ = interval.tick() => {}
                }
                tokio::select! {
                    _ = token.cancelled() => break,
                    res = checker.check() => {
                        debug!("Monitor {} {} check result: {:?}", monitor.id, monitor.name, res.status);

                        if let Err(e) = state.checks.create(
                            CreateMonitorCheckRequest {
                                monitor_id: monitor.id,
                                status: match res.status {
                                    Status::Up => shared::models::MonitorCheckStatus::Up,
                                    Status::Down => shared::models::MonitorCheckStatus::Down,
                                    Status::Unknown => shared::models::MonitorCheckStatus::Down, // Treat unknown as down for recording purposes
                                },
                                status_code: res.status_code,
                                response_time_ms: res.response_time_ms,
                                error_message: res.error.clone(),
                            }
                        ).await {
                            warn!("Failed to persist check for monitor {} {}: {}", monitor.id, monitor.name, e);
                        }

                        let mut guard = status.lock().unwrap_or_else(|e| e.into_inner());
                        match *guard {
                            Status::Up => {
                                if res.status == Status::Down {
                                    info!("Monitor went Down: {} {}", monitor.id, monitor.name);
                                }
                            },
                            Status::Down => {
                                if res.status == Status::Up {
                                    info!("Monitor went Up: {} {}", monitor.id, monitor.name);
                                }
                            },
                            Status::Unknown => {
                                info!("Initial status for monitor {} {}: {:?}", monitor.id, monitor.name, res.status);
                            },
                        }
                        *guard = res.status;
                    }
                }
            }
        });
    }
    pub async fn stop(&self) {
        debug!(
            "Stopping monitor worker for monitor: {} {}",
            self.monitor.id, self.monitor.name
        );
        self.cancellation_token.cancel();
    }

    pub async fn restart(&mut self) {
        self.stop().await;
        self.cancellation_token = CancellationToken::new();
        self.start().await;
    }
}
