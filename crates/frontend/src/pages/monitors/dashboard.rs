use crate::api::{Incident, MonitorCheckStatus};
use crate::components::{MonitorCard, StatsBar};
use super::DashboardData;
use yew::prelude::*;

fn fmt_relative(iso: &str) -> String {
    let js_date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(iso));
    let now = js_sys::Date::now();
    let diff = ((now - js_date.get_time()) / 1000.0) as i64;
    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        iso[..10].to_string()
    }
}

// ── Dashboard (loaded) ────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub(super) struct DashboardProps {
    pub data: DashboardData,
}

#[function_component(Dashboard)]
pub(super) fn dashboard(props: &DashboardProps) -> Html {
    let DashboardData { monitors, checks, incidents } = &props.data;

    let mut up_count = 0usize;
    let mut down_count = 0usize;
    let mut active_incidents: Vec<(usize, &Incident)> = vec![];

    for (i, (m_checks, m_incidents)) in checks.iter().zip(incidents.iter()).enumerate() {
        if let Some(latest) = m_checks.first() {
            if latest.status == MonitorCheckStatus::Up {
                up_count += 1;
            } else {
                down_count += 1;
            }
        }
        if let Some(inc) = m_incidents.iter().find(|i| i.resolved_at.is_none()) {
            active_incidents.push((i, inc));
        }
    }

    let incident_count = active_incidents.len();

    html! {
        <>
            <StatsBar
                total={monitors.len().to_string()}
                up={up_count.to_string()}
                down={down_count.to_string()}
                incidents={incident_count.to_string()}
            />

            { if !active_incidents.is_empty() {
                html! {
                    <div class="incidents-banner">
                        <svg class="incidents-banner-icon" viewBox="0 0 24 24" fill="none"
                            stroke="currentColor" stroke-width="2"
                            stroke-linecap="round" stroke-linejoin="round">
                            <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
                            <line x1="12" y1="9" x2="12" y2="13"/>
                            <line x1="12" y1="17" x2="12.01" y2="17"/>
                        </svg>
                        <div class="incidents-banner-body">
                            <div class="incidents-banner-title">
                                { format!("{} active incident{}", incident_count, if incident_count > 1 { "s" } else { "" }) }
                            </div>
                            { for active_incidents.iter().map(|(idx, inc)| {
                                let monitor = &monitors[*idx];
                                html! {
                                    <div class="incident-item">
                                        <span class="badge badge-down">{ "DOWN" }</span>
                                        <strong>{ &monitor.name }</strong>
                                        <span>{ format!("— down since {}", fmt_relative(&inc.started_at.to_rfc3339())) }</span>
                                    </div>
                                }
                            })}
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}

            <div class="section-header">
                <span class="section-title">{ "All monitors" }</span>
            </div>

            { if monitors.is_empty() {
                html! {
                    <div class="empty-state">
                        <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none"
                            stroke="currentColor" stroke-width="1.5">
                            <path d="M9 12l2 2 4-4M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                stroke-linecap="round" stroke-linejoin="round"/>
                        </svg>
                        <h3>{ "No monitors yet" }</h3>
                        <p>{ "Add your first monitor to start tracking uptime." }</p>
                        <a href="/create" class="btn btn-primary">{ "Add monitor" }</a>
                    </div>
                }
            } else {
                html! {
                    <div class="monitors-grid">
                        { for monitors.iter().enumerate().map(|(i, m)| html! {
                            <MonitorCard
                                monitor={m.clone()}
                                checks={checks[i].clone()}
                                incidents={incidents[i].clone()}
                            />
                        })}
                    </div>
                }
            }}
        </>
    }
}
