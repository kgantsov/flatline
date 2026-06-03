use std::collections::HashMap;
use std::sync::Arc;

use shared::models::Monitor;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{AppState, monitor::worker::MonitorWorker};

/// Clonable handle that gives API handlers control over running workers.
#[derive(Clone)]
pub struct EngineHandle {
    workers: Arc<Mutex<HashMap<Uuid, MonitorWorker>>>,
}

impl Default for EngineHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineHandle {
    pub fn new() -> Self {
        Self {
            workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn stop_monitor(&self, id: Uuid) {
        let mut workers = self.workers.lock().await;
        if let Some(worker) = workers.remove(&id) {
            worker.stop().await;
        }
    }

    pub async fn start_monitor(&self, state: AppState, monitor: Monitor) {
        if !monitor.enabled {
            return;
        }
        let id = monitor.id;
        let worker = MonitorWorker::new(state, monitor);
        worker.start().await;
        self.workers.lock().await.insert(id, worker);
    }

    /// Stops the existing worker (if any) and starts a fresh one if the monitor is enabled.
    pub async fn restart_monitor(&self, state: AppState, monitor: Monitor) {
        self.stop_monitor(monitor.id).await;
        self.start_monitor(state, monitor).await;
    }
}

pub struct MonitorEngine {
    state: AppState,
    handle: EngineHandle,
}

impl MonitorEngine {
    pub fn new(state: AppState, handle: EngineHandle) -> Self {
        Self { state, handle }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        let monitors = self.state.monitors.list().await?;

        for monitor in monitors {
            if !monitor.enabled {
                continue;
            }
            self.handle.start_monitor(self.state.clone(), monitor).await;
        }

        Ok(())
    }

    pub async fn stop(&mut self) {
        let mut workers = self.handle.workers.lock().await;
        for worker in workers.values() {
            worker.stop().await;
        }
        workers.clear();
    }
}
