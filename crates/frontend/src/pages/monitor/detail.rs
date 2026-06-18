use super::{PageData, Tab, charts};
use crate::api::{
    Incident, Monitor, MonitorCheck, MonitorCheckStatus, MonitorConfig, MonitorStats,
};
use crate::components::NotifLinker;
use crate::utils::{fmt_date, fmt_ms, monitor_url};
use yew::prelude::*;

fn monitor_method(m: &Monitor) -> String {
    match &m.config {
        MonitorConfig::Http { method, .. } => method
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "GET".into()),
    }
}

fn calc_streak(checks: &[MonitorCheck]) -> usize {
    if checks.is_empty() {
        return 0;
    }
    let status = &checks[0].status;
    checks.iter().take_while(|c| &c.status == status).count()
}

fn calc_p99(checks: &[MonitorCheck]) -> Option<u64> {
    if checks.is_empty() {
        return None;
    }
    let mut times: Vec<u64> = checks.iter().map(|c| c.response_time_ms).collect();
    times.sort_unstable();
    let idx = ((times.len() as f64 * 0.99) as usize).min(times.len() - 1);
    Some(times[idx])
}

/// Returns (mean_seconds, resolved_incident_count), or None if no resolved incidents.
fn calc_mttr(incidents: &[Incident]) -> Option<(u64, usize)> {
    let resolved: Vec<u64> = incidents
        .iter()
        .filter_map(|i| {
            let resolved = i.resolved_at?;
            let secs = (resolved - i.started_at).num_seconds();
            if secs > 0 { Some(secs as u64) } else { None }
        })
        .collect();
    if resolved.is_empty() {
        return None;
    }
    let mean = resolved.iter().sum::<u64>() / resolved.len() as u64;
    Some((mean, resolved.len()))
}

fn uptime_cls(pct: f64) -> &'static str {
    if pct >= 99.0 {
        "stat-value good"
    } else if pct < 95.0 {
        "stat-value bad"
    } else {
        "stat-value"
    }
}

fn fmt_downtime(total_secs: u64) -> String {
    if total_secs == 0 {
        return "0s".into();
    }
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

// ── MonitorDetail ─────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub(super) struct MonitorDetailProps {
    pub data: PageData,
    pub active_tab: Tab,
    pub on_tab_checks: Callback<MouseEvent>,
    pub on_tab_incidents: Callback<MouseEvent>,
    pub on_tab_notifications: Callback<MouseEvent>,
    pub on_reload: Callback<()>,
    /// Live status from SSE (overrides loaded check for the badge).
    pub live_status: Option<MonitorCheckStatus>,
    /// Live computed stats from SSE (7d/30d/90d uptime + latency percentiles).
    pub live_stats: Option<MonitorStats>,
}

#[function_component(MonitorDetail)]
pub(super) fn monitor_detail(props: &MonitorDetailProps) -> Html {
    let PageData {
        monitor,
        checks,
        incidents,
        notifications,
        channels,
    } = &props.data;

    let latest = checks.first().map(|c| &c.status);
    // Prefer SSE live status for the badge; fall back to last loaded check.
    let effective_status = props.live_status.as_ref().or(latest);
    let effective_str = effective_status
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".into());

    let badge_cls = format!(
        "status-badge {}",
        if !monitor.enabled {
            "unknown"
        } else {
            &effective_str
        }
    );
    let badge_label = if !monitor.enabled {
        "Paused"
    } else if effective_status == Some(&MonitorCheckStatus::Up) {
        "Operational"
    } else if effective_status == Some(&MonitorCheckStatus::Down) {
        "Down"
    } else {
        "Unknown"
    };

    let up_count = checks
        .iter()
        .filter(|c| c.status == MonitorCheckStatus::Up)
        .count();
    let uptime_pct = if checks.is_empty() {
        None
    } else {
        Some(up_count as f64 / checks.len() as f64 * 100.0)
    };
    let avg_resp = if checks.is_empty() {
        None
    } else {
        Some(checks.iter().map(|c| c.response_time_ms).sum::<u64>() / checks.len() as u64)
    };
    let p99 = calc_p99(checks);
    let streak = calc_streak(checks);
    let active_incident = incidents.iter().find(|i| i.resolved_at.is_none());

    let uptime_cls_val = uptime_pct.map(uptime_cls).unwrap_or("stat-value");

    let streak_cls = if effective_status == Some(&MonitorCheckStatus::Up) {
        "stat-value good"
    } else {
        "stat-value bad"
    };

    let tab_cls = |t: &Tab| {
        if *t == props.active_tab {
            "tab active"
        } else {
            "tab"
        }
    };

    html! {
        <>
            <div class="breadcrumb">
                <a href="/">{ "Monitors" }</a>
                <span class="breadcrumb-sep">{ "/" }</span>
                <span>{ &monitor.name }</span>
            </div>

            <div class="monitor-hero">
                <div class="monitor-hero-left">
                    <div class="monitor-title">
                        <h1>{ &monitor.name }</h1>
                        <div class="monitor-url-hero">{ monitor_url(monitor) }</div>
                        <div class="monitor-meta">
                            <span class="meta-chip">{ monitor_method(monitor) }</span>
                            <span class="meta-chip">{ format!("every {}s", monitor.interval) }</span>
                            <span class="meta-chip">{ format!("timeout {}s", monitor.timeout) }</span>
                            <span class="meta-chip">{ format!("{} retries", monitor.retries) }</span>
                            { if !monitor.enabled {
                                html! { <span class="meta-chip" style="color:var(--text-muted)">{ "paused" }</span> }
                            } else { html! {} }}
                        </div>
                    </div>
                </div>
                <div style="display:flex;align-items:flex-start;gap:12px;flex-wrap:wrap">
                    <span class={badge_cls}>
                        <span class={format!("status-dot-sm {}", effective_str)}></span>
                        { badge_label }
                    </span>
                    { if let Some(inc) = active_incident {
                        html! {
                            <span style="font-size:12.5px;color:var(--down);margin-top:8px">
                                { format!("Incident ongoing — since {}", fmt_date(&inc.started_at.to_rfc3339())) }
                            </span>
                        }
                    } else { html! {} }}
                </div>
            </div>

            // ── Recent checks stats (from HTTP load) ──────────────────────────
            <div class="stats-row">
                <div class="stat-card">
                    <div class="stat-label">{ format!("Uptime (last {})", checks.len()) }</div>
                    <div class={uptime_cls_val}>
                        { uptime_pct.map(|p| format!("{:.2}%", p)).unwrap_or_else(|| "—".into()) }
                    </div>
                    <div class="stat-sub">{ "recent checks" }</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">{ "Avg response" }</div>
                    <div class="stat-value">
                        { avg_resp.map(fmt_ms).unwrap_or_else(|| "—".into()) }
                    </div>
                    <div class="stat-sub">
                        { p99.map(|p| format!("p99 {}", fmt_ms(p))).unwrap_or_else(|| "no data".into()) }
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">{ "Current streak" }</div>
                    <div class={streak_cls}>{ streak.to_string() }</div>
                    <div class="stat-sub">
                        { if effective_status == Some(&MonitorCheckStatus::Up) { "checks up" } else { "checks down" } }
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">{ "Total incidents" }</div>
                    <div class="stat-value">{ incidents.len().to_string() }</div>
                    <div class="stat-sub">
                        { if active_incident.is_some() { "1 active now" } else { "none active" } }
                    </div>
                </div>
            </div>

            // ── Live rolling-window stats from SSE ────────────────────────────
            { if let Some(ls) = &props.live_stats {
                html! {
                    <div class="stats-row">
                        <div class="stat-card">
                            <div class="stat-label">{ "7d uptime" }</div>
                            <div class={uptime_cls(ls.uptime_7d * 100.0)}>
                                { format!("{:.2}%", ls.uptime_7d * 100.0) }
                            </div>
                            <div class="stat-sub">
                                { format!("{} downtime · p99 {}", fmt_downtime(ls.downtime_seconds_7d), fmt_ms(ls.p99_7d)) }
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-label">{ "30d uptime" }</div>
                            <div class={uptime_cls(ls.uptime_30d * 100.0)}>
                                { format!("{:.2}%", ls.uptime_30d * 100.0) }
                            </div>
                            <div class="stat-sub">
                                { format!("{} downtime · p99 {}", fmt_downtime(ls.downtime_seconds_30d), fmt_ms(ls.p99_30d)) }
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-label">{ "90d uptime" }</div>
                            <div class={uptime_cls(ls.uptime_90d * 100.0)}>
                                { format!("{:.2}%", ls.uptime_90d * 100.0) }
                            </div>
                            <div class="stat-sub">
                                { format!("{} downtime · p99 {}", fmt_downtime(ls.downtime_seconds_90d), fmt_ms(ls.p99_90d)) }
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-label">{ "MTTR" }</div>
                            { if let Some((mean_secs, count)) = calc_mttr(incidents) {
                                html! {
                                    <>
                                        <div class="stat-value">{ fmt_downtime(mean_secs) }</div>
                                        <div class="stat-sub">{ format!("avg over {} incidents", count) }</div>
                                    </>
                                }
                            } else {
                                html! {
                                    <>
                                        <div class="stat-value">{ "—" }</div>
                                        <div class="stat-sub">{ "no resolved incidents" }</div>
                                    </>
                                }
                            }}
                        </div>
                    </div>
                }
            } else { html! {} }}

            <div class="card">
                <div class="card-header">
                    <span class="card-title">{ "Response time" }</span>
                    <span style="font-size:12px;color:var(--text-muted)">
                        { format!("Last {} checks", checks.len()) }
                    </span>
                </div>
                <div class="card-body">
                    { charts::response_chart(checks) }
                </div>
            </div>

            <div class="card">
                <div class="tabs">
                    <button class={tab_cls(&Tab::Checks)} onclick={props.on_tab_checks.clone()}>
                        { "Recent checks" }
                    </button>
                    <button class={tab_cls(&Tab::Incidents)} onclick={props.on_tab_incidents.clone()}>
                        { "Incidents" }
                    </button>
                    <button class={tab_cls(&Tab::Notifications)} onclick={props.on_tab_notifications.clone()}>
                        { "Notifications" }
                    </button>
                </div>

                { match props.active_tab {
                    Tab::Checks => html! {
                        <div>{ charts::checks_table(checks) }</div>
                    },
                    Tab::Incidents => html! {
                        <div>{ charts::incidents_list(incidents) }</div>
                    },
                    Tab::Notifications => html! {
                        <div class="card-body">
                            <NotifLinker
                                monitor_id={monitor.id.to_string()}
                                notifications={notifications.clone()}
                                channels={channels.clone()}
                                on_reload={props.on_reload.clone()}
                            />
                        </div>
                    },
                }}
            </div>
        </>
    }
}
