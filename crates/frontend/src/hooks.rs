use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::{EventSource, MessageEvent};
use yew::prelude::*;

use crate::api::{Incident, MonitorCheck, MonitorCheckStatus, MonitorStats, SseEvent};

#[derive(Default, Clone, PartialEq)]
pub struct SseStats {
    pub stats: HashMap<String, MonitorStats>,
    pub latest_status: HashMap<String, MonitorCheckStatus>,
    pub latest_response_ms: HashMap<String, u64>,
    /// Most-recent checks per monitor, newest first, capped at 100.
    pub recent_checks: HashMap<String, Vec<MonitorCheck>>,
    /// Live incident mutations per monitor (opened/resolved via SSE).
    pub live_incidents: HashMap<String, Vec<Incident>>,
    pub connected: bool,
}

/// Connects to /api/v1/stats/stream and returns a handle that updates on every SSE event.
///
/// The server sends all events as plain `message` events; the JSON `"type"` field
/// discriminates the variant. We use `set_onmessage` which is the most reliable
/// wasm-bindgen path for SSE.
#[hook]
pub fn use_sse_stats() -> UseStateHandle<SseStats> {
    let state = use_state(SseStats::default);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            let inner = Rc::new(RefCell::new(SseStats::default()));

            let es = EventSource::new("/api/v1/stats/stream")
                .expect("EventSource::new failed");

            let onopen = {
                let state = state.clone();
                let inner = inner.clone();
                Closure::<dyn Fn()>::wrap(Box::new(move || {
                    inner.borrow_mut().connected = true;
                    state.set(inner.borrow().clone());
                }))
            };
            es.set_onopen(Some(onopen.as_ref().unchecked_ref()));

            let onmessage = {
                let state = state.clone();
                let inner = inner.clone();
                Closure::<dyn Fn(MessageEvent)>::wrap(Box::new(move |e: MessageEvent| {
                    let data = e.data().as_string().unwrap_or_default();
                    let Ok(event) = serde_json::from_str::<SseEvent>(&data) else {
                        return;
                    };
                    {
                        let mut s = inner.borrow_mut();
                        match event {
                            SseEvent::StatsUpdate { monitor_id, stats } => {
                                s.stats.insert(monitor_id.to_string(), stats);
                            }
                            SseEvent::CheckResult {
                                monitor_id,
                                status,
                                status_code,
                                response_time_ms,
                                error_message,
                                checked_at,
                            } => {
                                let mid = monitor_id.to_string();
                                s.latest_status.insert(mid.clone(), status.clone());
                                s.latest_response_ms.insert(mid.clone(), response_time_ms);

                                let check = MonitorCheck {
                                    id: uuid::Uuid::nil(),
                                    monitor_id,
                                    status,
                                    status_code,
                                    response_time_ms,
                                    error_message,
                                    checked_at,
                                };
                                let list = s.recent_checks.entry(mid).or_default();
                                list.insert(0, check);
                                list.truncate(100);
                            }
                            SseEvent::IncidentOpened { monitor_id, incident_id, started_at } => {
                                let mid = monitor_id.to_string();
                                let inc = Incident {
                                    id: incident_id,
                                    monitor_id,
                                    started_at,
                                    resolved_at: None,
                                };
                                let list = s.live_incidents.entry(mid).or_default();
                                list.insert(0, inc);
                            }
                            SseEvent::IncidentResolved { monitor_id, incident_id, started_at, resolved_at } => {
                                let mid = monitor_id.to_string();
                                let list = s.live_incidents.entry(mid).or_default();
                                if let Some(existing) = list.iter_mut().find(|i| i.id == incident_id) {
                                    existing.resolved_at = Some(resolved_at);
                                } else {
                                    list.insert(0, Incident {
                                        id: incident_id,
                                        monitor_id,
                                        started_at,
                                        resolved_at: Some(resolved_at),
                                    });
                                }
                            }
                        }
                    }
                    state.set(inner.borrow().clone());
                }))
            };
            es.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

            move || {
                es.close();
                drop(onopen);
                drop(onmessage);
            }
        });
    }

    state
}
