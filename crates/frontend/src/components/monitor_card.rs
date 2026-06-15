use crate::api::{Incident, Monitor, MonitorCheck};
use crate::components::Sparkline;
use crate::utils::{fmt_ms, monitor_url, uptime_class};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MonitorCardProps {
    pub monitor: Monitor,
    pub checks: Vec<MonitorCheck>,
    pub incidents: Vec<Incident>,
}

#[function_component(MonitorCard)]
pub fn monitor_card(props: &MonitorCardProps) -> Html {
    let m = &props.monitor;
    let checks = &props.checks;

    let latest_status = checks.first().map(|c| c.status.as_str()).unwrap_or("unknown");
    let dot_class = if !m.enabled {
        "status-dot unknown"
    } else if latest_status == "up" {
        "status-dot up"
    } else if latest_status == "down" {
        "status-dot down"
    } else {
        "status-dot unknown"
    };

    let up_checks = checks.iter().filter(|c| c.status == "up").count();
    let uptime_pct = if checks.is_empty() {
        None
    } else {
        Some(up_checks as f64 / checks.len() as f64 * 100.0)
    };

    let avg_response = if checks.is_empty() {
        None
    } else {
        let total: u64 = checks.iter().map(|c| c.response_time_ms).sum();
        Some(total / checks.len() as u64)
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
                            <div class={cls}>{ format!("{:.1}%", pct) }</div>
                            <div class="uptime-label">{ "uptime" }</div>
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
                            <div class="response-label">{ "avg resp." }</div>
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
