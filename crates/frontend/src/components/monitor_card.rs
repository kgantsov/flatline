use crate::api::{Incident, Monitor, MonitorCheck, MonitorCheckStatus, MonitorStats};
use crate::components::Sparkline;
use crate::utils::{fmt_ms, monitor_url, uptime_class};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MonitorCardProps {
    pub monitor: Monitor,
    pub checks: Vec<MonitorCheck>,
    pub incidents: Vec<Incident>,
    /// Live status from the SSE stream (overrides check history for the dot).
    pub live_status: Option<MonitorCheckStatus>,
    /// Live stats from the SSE stream (uptime_30d, p50_30d, etc.).
    pub live_stats: Option<MonitorStats>,
}

#[function_component(MonitorCard)]
pub fn monitor_card(props: &MonitorCardProps) -> Html {
    let m = &props.monitor;
    let checks = &props.checks;

    // Prefer live SSE status for the indicator dot; fall back to latest loaded check.
    let latest_from_checks = checks.first().map(|c| &c.status);
    let effective_status = props.live_status.as_ref().or(latest_from_checks);

    let dot_class = if !m.enabled {
        "status-dot unknown"
    } else if effective_status == Some(&MonitorCheckStatus::Up) {
        "status-dot up"
    } else if effective_status == Some(&MonitorCheckStatus::Down) {
        "status-dot down"
    } else {
        "status-dot unknown"
    };

    // Use SSE 30d uptime when available; otherwise compute from loaded checks.
    let (uptime_pct, uptime_label) = if let Some(s) = &props.live_stats {
        (Some(s.uptime_30d * 100.0), "30d")
    } else if checks.is_empty() {
        (None, "uptime")
    } else {
        let up = checks.iter().filter(|c| c.status == MonitorCheckStatus::Up).count();
        (Some(up as f64 / checks.len() as f64 * 100.0), "uptime")
    };

    // Use SSE p50 latency when available; otherwise compute avg from loaded checks.
    let (avg_response, response_label) = if let Some(s) = &props.live_stats {
        (Some(s.p50_30d), "p50 30d")
    } else if checks.is_empty() {
        (None, "avg resp.")
    } else {
        let total: u64 = checks.iter().map(|c| c.response_time_ms).sum();
        (Some(total / checks.len() as u64), "avg resp.")
    };

    let card_class = if m.enabled { "monitor-card" } else { "monitor-card disabled" };
    let href = format!("/monitors/{}", m.id);

    html! {
        <a class={card_class} href={href}>
            <span class={dot_class}></span>

            <div class="monitor-info">
                <div class="monitor-name">{ &m.name }</div>
                <div class="monitor-url">{ monitor_url(m) }</div>
            </div>

            <div class="monitor-uptime">
                { if let Some(pct) = uptime_pct {
                    let cls = format!("uptime-pct {}", uptime_class(pct));
                    html! {
                        <>
                            <div class={cls}>{ format!("{:.2}%", pct) }</div>
                            <div class="uptime-label">{ uptime_label }</div>
                        </>
                    }
                } else {
                    html! { <div class="uptime-pct" style="color:var(--text-muted)">{ "—" }</div> }
                }}
            </div>

            <Sparkline checks={checks.clone()} />

            <div class="monitor-response">
                { if let Some(ms) = avg_response {
                    html! {
                        <>
                            <div class="response-time">{ fmt_ms(ms) }</div>
                            <div class="response-label">{ response_label }</div>
                        </>
                    }
                } else {
                    html! { <div class="response-time" style="color:var(--text-muted)">{ "—" }</div> }
                }}
            </div>

            <div class="monitor-interval">{ format!("every {}s", m.interval) }</div>
        </a>
    }
}
