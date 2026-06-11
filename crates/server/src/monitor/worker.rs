use std::sync::{Arc, Mutex};

use crate::notify::NotificationEvent;
use chrono::Utc;
use shared::{
    api::CreateMonitorCheckRequest,
    models::{Monitor, MonitorConfig, NotificationChannelConfig},
};
use tokio::time::{self, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    AppState,
    monitor::{
        checker::{Checker, Status},
        http::HttpChecker,
    },
    notify::{Notifier, slack::SlackNotifier, webhook::WebhookNotifier},
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
                Duration::from_secs(monitor.timeout as u64),
            )),
        }
    }

    fn build_notifier(config: &NotificationChannelConfig) -> Box<dyn Notifier> {
        match config {
            NotificationChannelConfig::Webhook { url } => {
                Box::new(WebhookNotifier::new(None, url.clone()))
            }
            NotificationChannelConfig::Slack { webhook_url } => {
                Box::new(SlackNotifier::new(webhook_url.clone()))
            }
        }
    }

    async fn get_notifiers(state: AppState, monitor_id: Uuid) -> Vec<Box<dyn Notifier>> {
        let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();
        if let Ok(notifications) = state
            .monitor_notifications
            .list_for_monitor(monitor_id)
            .await
        {
            for n in notifications {
                if let Ok(channel) = state.notification_channels.get(n.channel_id).await {
                    notifiers.push(Self::build_notifier(&channel.config));
                }
            }
        }
        notifiers
    }

    pub async fn start(&self) {
        debug!(
            "Starting monitor worker for monitor: {} {}",
            self.monitor.id, self.monitor.name
        );

        let monitor_id = self.monitor.id;
        if let Some(last_check) = self
            .state
            .checks
            .list_for_monitor(monitor_id, 1, None)
            .await
            .ok()
            .and_then(|mut checks| checks.pop())
        {
            debug!(
                "Last check for monitor {} {}: status={:?}, status_code={:?}, response_time_ms={}, error={:?}",
                self.monitor.id,
                self.monitor.name,
                last_check.status,
                last_check.status_code,
                last_check.response_time_ms,
                last_check.error_message
            );
            let mut guard = self.status.lock().unwrap_or_else(|e| e.into_inner());
            *guard = match last_check.status {
                shared::models::MonitorCheckStatus::Up => Status::Up,
                shared::models::MonitorCheckStatus::Down => Status::Down,
            };
        }

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

                        let (went_down, went_up) = {
                            let mut guard = status.lock().unwrap_or_else(|e| e.into_inner());
                            let prev = guard.clone();
                            *guard = res.status.clone();

                            match prev {
                                Status::Up => (res.status == Status::Down, false),
                                Status::Down => (false, res.status == Status::Up),
                                Status::Unknown => {
                                    info!("Initial status for monitor {} {}: {:?}", monitor.id, monitor.name, res.status);
                                    (false, false)
                                },
                            }
                        };

                        let notifiers: Vec<Box<dyn Notifier>> = Self::get_notifiers(
                            state.clone(), monitor.id
                        ).await;

                        if went_down {
                            info!("Monitor went Down: {} {}", monitor.id, monitor.name);
                            state.incidents.open(monitor.id, Utc::now()).await.ok();

                            let event = NotificationEvent::MonitorDown {
                                  monitor: monitor.clone(),
                                  checked_at: Utc::now(),
                                  error: res.error.clone().unwrap_or_default(),
                            };

                            for notifier in &notifiers {
                                info!(
                                    "Sending down notification for monitor {} {}: {:?}",
                                    monitor.id,
                                    monitor.name,
                                    event
                                );
                                if let Err(e) = notifier.send(event.clone()).await {
                                    warn!(
                                        "Failed to send notification for monitor {} {}: {}",
                                        monitor.id,
                                        monitor.name,
                                        e
                                    );
                                }
                            }
                        }

                        if went_up {
                            info!("Monitor went Up: {} {}", monitor.id, monitor.name);
                            if let Ok(Some(incident)) = state.incidents.get_open_for_monitor(monitor.id).await {
                                state.incidents.resolve(incident.id, Utc::now()).await.ok();

                                let event = NotificationEvent::MonitorRecovered {
                                    monitor: monitor.clone(),
                                    incident,
                                };

                                for notifier in &notifiers {
                                    info!(
                                        "Sending recovery notification for monitor {} {}: {:?}",
                                        monitor.id,
                                        monitor.name,
                                        event
                                    );
                                    if let Err(e) = notifier.send(event.clone()).await {
                                        warn!(
                                            "Failed to send notification for monitor {} {}: {}",
                                            monitor.id,
                                            monitor.name,
                                            e
                                        );
                                    }
                                }
                            }
                        }
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
