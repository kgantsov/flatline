use std::sync::{Arc, Mutex};

use crate::monitor::checker::CheckOutcome;
use anyhow::Result;
use chrono::{Days, Utc};
use shared::{
    api::CreateMonitorCheckRequest,
    models::{Monitor, MonitorConfig, MonitorStats, NotificationChannelConfig, SseEvent},
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
    notify::{
        NotificationEvent, Notifier, discord::DiscordNotifier, slack::SlackNotifier,
        telegram::TelegramNotifier, webhook::WebhookNotifier,
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
                ref headers,
                ref body,
            } => Box::new(HttpChecker::new(
                url.clone(),
                method.clone(),
                expected_status.clone(),
                Duration::from_secs(monitor.timeout as u64),
                headers.clone(),
                body.clone(),
            )),
        }
    }

    fn build_notifier(config: &NotificationChannelConfig) -> Box<dyn Notifier> {
        match config {
            NotificationChannelConfig::Webhook { url } => {
                Box::new(WebhookNotifier::new(None, url.clone()))
            }
            NotificationChannelConfig::Slack { url } => Box::new(SlackNotifier::new(url.clone())),
            NotificationChannelConfig::Telegram { url, chat_id } => {
                Box::new(TelegramNotifier::new(url.clone(), chat_id.clone()))
            }
            NotificationChannelConfig::Discord { url } => {
                Box::new(DiscordNotifier::new(url.clone()))
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

        if let Err(err) = Self::update_stats(&self.state, &self.monitor).await {
            warn!(
                "Failed to update stats for monitor {} {}: {}",
                self.monitor.id, self.monitor.name, err
            );
        };

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
            let mut consecutive_failures: u32 = 0;
            loop {
                tokio::select! {
                    _ = token.cancelled() => break,
                    _ = interval.tick() => {}
                }
                tokio::select! {
                    _ = token.cancelled() => break,
                    res = checker.check() => {
                        debug!(
                            "Monitor {} {} check result: {:?}", monitor.id, monitor.name, res.status,
                        );

                        if let Err(err) = Self::record_result(
                            &state,
                            &monitor,
                            Arc::clone(&status),
                            &res,
                            &mut consecutive_failures
                        ).await {
                                warn!(
                                    "Failed to record check result for monitor {} {}: {}",
                                    monitor.id,
                                    monitor.name,
                                    err
                                );
                        };

                        if let Err(err) = Self::update_stats(&state, &monitor).await {
                            warn!(
                                "Failed to update stats for monitor {} {}: {}",
                                monitor.id,
                                monitor.name,
                                err
                            );
                        };
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

    async fn update_stats(state: &AppState, monitor: &Monitor) -> Result<()> {
        let now = Utc::now();
        let id = monitor.id;
        let created_at = monitor.created_at;

        let (uptime_7d, uptime_30d, uptime_90d, p_7d, p_30d, p_90d) = tokio::try_join!(
            state
                .incidents
                .uptime_percentage(id, created_at, now - Days::new(7)),
            state
                .incidents
                .uptime_percentage(id, created_at, now - Days::new(30)),
            state
                .incidents
                .uptime_percentage(id, created_at, now - Days::new(90)),
            state.incidents.latency_percentiles(id, now - Days::new(7)),
            state.incidents.latency_percentiles(id, now - Days::new(30)),
            state.incidents.latency_percentiles(id, now - Days::new(90)),
        )?;

        let (uptime_7d, downtime_seconds_7d) = uptime_7d.unwrap_or((0.0, 0));
        let (uptime_30d, downtime_seconds_30d) = uptime_30d.unwrap_or((0.0, 0));
        let (uptime_90d, downtime_seconds_90d) = uptime_90d.unwrap_or((0.0, 0));
        let (p50_7d, p95_7d, p99_7d) = p_7d.map_or((0, 0, 0), |p| (p.p50_ms, p.p95_ms, p.p99_ms));
        let (p50_30d, p95_30d, p99_30d) =
            p_30d.map_or((0, 0, 0), |p| (p.p50_ms, p.p95_ms, p.p99_ms));
        let (p50_90d, p95_90d, p99_90d) =
            p_90d.map_or((0, 0, 0), |p| (p.p50_ms, p.p95_ms, p.p99_ms));

        let stats = MonitorStats {
            uptime_7d,
            uptime_30d,
            uptime_90d,
            downtime_seconds_7d,
            downtime_seconds_30d,
            downtime_seconds_90d,
            p50_7d,
            p95_7d,
            p99_7d,
            p50_30d,
            p95_30d,
            p99_30d,
            p50_90d,
            p95_90d,
            p99_90d,
        };
        state.stats.insert(id, stats.clone());
        let _ = state.event_tx.send(SseEvent::StatsUpdate {
            monitor_id: id,
            stats,
        });

        Ok(())
    }

    async fn record_result(
        state: &AppState,
        monitor: &Monitor,
        status: Arc<Mutex<Status>>,
        res: &CheckOutcome,
        consecutive_failures: &mut u32,
    ) -> Result<()> {
        let check_status = match res.status {
            Status::Up => shared::models::MonitorCheckStatus::Up,
            Status::Down | Status::Unknown => shared::models::MonitorCheckStatus::Down,
        };
        let checked_at = Utc::now();

        if let Err(e) = state
            .checks
            .create(CreateMonitorCheckRequest {
                monitor_id: monitor.id,
                status: check_status.clone(),
                status_code: res.status_code,
                response_time_ms: res.response_time_ms,
                error_message: res.error.clone(),
            })
            .await
        {
            warn!(
                "Failed to persist check for monitor {} {}: {}",
                monitor.id, monitor.name, e
            );
        }

        let _ = state.event_tx.send(SseEvent::CheckResult {
            monitor_id: monitor.id,
            status: check_status,
            status_code: res.status_code,
            response_time_ms: res.response_time_ms,
            error_message: res.error.clone(),
            checked_at,
        });

        let (went_down, went_up) =
            Self::validate_check_result(monitor, status, res, consecutive_failures);

        if went_down {
            info!("Monitor went Down: {} {}", monitor.id, monitor.name);
            if let Ok(incident) = state.incidents.open(monitor.id, checked_at).await {
                let _ = state.event_tx.send(SseEvent::IncidentOpened {
                    monitor_id: monitor.id,
                    incident_id: incident.id,
                    started_at: incident.started_at,
                });
            }

            Self::notify(
                state,
                monitor,
                NotificationEvent::MonitorDown {
                    monitor: monitor.clone(),
                    checked_at,
                    error: res.error.clone().unwrap_or_default(),
                },
            )
            .await?;
        }

        if went_up {
            info!("Monitor went Up: {} {}", monitor.id, monitor.name);
            if let Ok(Some(incident)) = state.incidents.get_open_for_monitor(monitor.id).await {
                if let Ok(resolved) = state.incidents.resolve(incident.id, checked_at).await {
                    let _ = state.event_tx.send(SseEvent::IncidentResolved {
                        monitor_id: monitor.id,
                        incident_id: resolved.id,
                        started_at: resolved.started_at,
                        resolved_at: resolved.resolved_at.unwrap_or(checked_at),
                    });
                }

                Self::notify(
                    state,
                    monitor,
                    NotificationEvent::MonitorRecovered {
                        monitor: monitor.clone(),
                        incident,
                    },
                )
                .await?;
            }
        }

        Ok(())
    }

    fn validate_check_result(
        monitor: &Monitor,
        status: Arc<Mutex<Status>>,
        res: &CheckOutcome,
        consecutive_failures: &mut u32,
    ) -> (bool, bool) {
        let (went_down, went_up) = {
            let mut guard = status.lock().unwrap_or_else(|e| e.into_inner());
            let prev = guard.clone();

            match res.status {
                Status::Up => {
                    *consecutive_failures = 0;
                    *guard = Status::Up;
                    match prev {
                        Status::Down => (false, true),
                        Status::Unknown => {
                            info!(
                                "Initial status for monitor {} {}: Up",
                                monitor.id, monitor.name
                            );
                            (false, false)
                        }
                        Status::Up => (false, false),
                    }
                }
                Status::Down | Status::Unknown => {
                    *consecutive_failures += 1;
                    let threshold = monitor.retries + 1;

                    match prev {
                        Status::Up => {
                            if *consecutive_failures >= threshold {
                                *guard = Status::Down;
                                (true, false)
                            } else {
                                debug!(
                                    "Monitor {} {} failed ({}/{}), within retry budget",
                                    monitor.id, monitor.name, consecutive_failures, threshold
                                );
                                (false, false)
                            }
                        }
                        Status::Down => (false, false),
                        Status::Unknown => {
                            if *consecutive_failures >= threshold {
                                *guard = Status::Down;
                                info!(
                                    "Initial status for monitor {} {}: Down",
                                    monitor.id, monitor.name
                                );
                            }
                            (false, false)
                        }
                    }
                }
            }
        };

        (went_down, went_up)
    }

    async fn notify(state: &AppState, monitor: &Monitor, event: NotificationEvent) -> Result<()> {
        let notifiers: Vec<Box<dyn Notifier>> =
            Self::get_notifiers(state.clone(), monitor.id).await;

        for notifier in &notifiers {
            match event {
                NotificationEvent::MonitorDown { .. } => info!(
                    "Sending down notification for monitor {} {}: {:?}",
                    monitor.id, monitor.name, event
                ),
                NotificationEvent::MonitorRecovered { .. } => info!(
                    "Sending recovery notification for monitor {} {}: {:?}",
                    monitor.id, monitor.name, event
                ),
            }
            if let Err(e) = notifier.send(event.clone()).await {
                warn!(
                    "Failed to send notification for monitor {} {}: {}",
                    monitor.id, monitor.name, e
                );
            }
        }
        Ok(())
    }
}
