use std::sync::{Arc, Mutex};

use shared::models::{Monitor, MonitorConfig};
use tokio::time::{self, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::{
    AppState,
    monitor::{
        checker::{Checker, Status},
        http::HttpChecker,
    },
};

pub struct MonitorWorker {
    _state: AppState,
    monitor: Monitor,
    cancellation_token: CancellationToken,
    status: Arc<Mutex<Status>>,
}

impl MonitorWorker {
    pub fn new(state: AppState, monitor: Monitor) -> Self {
        Self {
            _state: state,
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
        let token = self.cancellation_token.clone();
        let monitor = self.monitor.clone();

        let interval_secs = Duration::from_secs(self.monitor.interval as u64);
        let mut interval = time::interval_at(time::Instant::now() + interval_secs, interval_secs);

        let status = Arc::clone(&self.status);

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
                        let mut guard = status.lock().unwrap();
                        debug!("Monitor {} {} check result: {:?}", monitor.id, monitor.name, res.status);
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
