use crate::api::{Incident, MonitorCheck, MonitorCheckStatus};
use crate::utils::{fmt_date, fmt_duration, fmt_ms};
use yew::prelude::*;

pub(super) fn response_chart(checks: &[MonitorCheck]) -> Html {
    if checks.is_empty() {
        return html! { <div class="empty-table">{ "No check data yet." }</div> };
    }

    let data: Vec<_> = checks.iter().rev().collect();
    let max_ms = data
        .iter()
        .map(|c| c.response_time_ms)
        .max()
        .unwrap_or(1)
        .max(1);
    let n = data.len();
    let vw = 600.0f64;
    let vh = 160.0f64;
    let slot = vw / n as f64;
    let bar_w = (slot - 1.5).max(2.0);

    let bars: Vec<Html> = data
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let x = i as f64 * slot;
            let bar_h = (c.response_time_ms as f64 / max_ms as f64 * vh).max(2.0);
            let y = vh - bar_h;
            let fill = if c.status == MonitorCheckStatus::Up {
                "rgba(99,102,241,.75)"
            } else {
                "rgba(239,68,68,.8)"
            };
            let checked_at_str = c.checked_at.to_rfc3339();
            let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(&checked_at_str));
            let label = format!("{:02}:{:02}", d.get_hours(), d.get_minutes());
            html! {
                <rect
                    x={format!("{:.1}", x)} y={format!("{:.1}", y)}
                    width={format!("{:.1}", bar_w)} height={format!("{:.1}", bar_h)}
                    fill={fill} rx="1">
                    <title>{ format!("{} – {}ms", label, c.response_time_ms) }</title>
                </rect>
            }
        })
        .collect();

    html! {
        <div class="chart-wrap">
            <svg viewBox={format!("0 0 {} {}", vw, vh)}
                 preserveAspectRatio="none"
                 style="width:100%;height:100%;display:block">
                { for bars }
            </svg>
        </div>
    }
}

pub(super) fn checks_table(checks: &[MonitorCheck], timeout_secs: u64) -> Html {
    if checks.is_empty() {
        return html! { <div class="empty-table">{ "No checks recorded yet." }</div> };
    }

    let timeout_ms = (timeout_secs * 1000).max(1);

    let rows: Vec<Html> = checks.iter().take(50).map(|c| {
        let bar_w = ((c.response_time_ms as f64 / timeout_ms as f64) * 120.0).min(120.0).round() as u64;
        let status_str = c.status.to_string();
        let bar_color = if c.status == MonitorCheckStatus::Down { "var(--down)" } else { "var(--accent)" };
        let pill_cls = format!("status-pill {}", status_str);
        html! {
            <tr>
                <td style="width:70px"><span class={pill_cls}>{ &status_str }</span></td>
                <td class="code-cell" style="width:52px">
                    { c.status_code.map(|s| s.to_string()).unwrap_or_else(|| "—".into()) }
                </td>
                <td>
                    <div class="resp-bar-wrap">
                        <div class="resp-bar"
                             style={format!("width:{}px;background:{}", bar_w, bar_color)}>
                        </div>
                        <span class="code-cell" style="margin-left:auto">{ fmt_ms(c.response_time_ms) }</span>
                    </div>
                </td>
                <td class="text-muted" style="font-size:12.5px;width:140px;white-space:nowrap">{ fmt_date(&c.checked_at.to_rfc3339()) }</td>
                <td style="font-size:12px;color:var(--down);max-width:220px">
                    { c.error_message.clone().unwrap_or_default() }
                </td>
            </tr>
        }
    }).collect();

    html! {
        <table class="checks-table">
            <thead>
                <tr>
                    <th style="width:70px">{ "Status" }</th>
                    <th style="width:52px">{ "Code" }</th>
                    <th>{ "Response time" }</th>
                    <th style="width:140px">{ "Time" }</th>
                    <th>{ "Error" }</th>
                </tr>
            </thead>
            <tbody>{ for rows }</tbody>
        </table>
    }
}

pub(super) fn incidents_list(incidents: &[Incident]) -> Html {
    if incidents.is_empty() {
        return html! { <div class="empty-table">{ "No incidents recorded. Great job!" }</div> };
    }

    let rows: Vec<Html> = incidents.iter().map(|inc| {
        let active = inc.resolved_at.is_none();
        let dot_cls = if active { "incident-dot active" } else { "incident-dot resolved" };
        let started_str = inc.started_at.to_rfc3339();
        let resolved_str = inc.resolved_at.as_ref().map(|d| d.to_rfc3339());
        let dur = fmt_duration(&started_str, resolved_str.as_deref());

        html! {
            <div class="incident-row">
                <div class={dot_cls}></div>
                <div>
                    <div class="incident-title">
                        { if active {
                            html! { <span style="color:var(--down);font-weight:600">{ "Ongoing outage" }</span> }
                        } else {
                            html! { "Resolved incident" }
                        }}
                    </div>
                    <div class="incident-meta">
                        { format!("Started {}", fmt_date(&started_str)) }
                        { resolved_str.as_ref().map(|r| format!(" — Resolved {}", fmt_date(r))).unwrap_or_default() }
                    </div>
                </div>
                <div class="incident-duration">
                    { if active { "ongoing".into() } else { dur } }
                </div>
            </div>
        }
    }).collect();

    html! {
        <div class="card-body" style="padding:0 20px">
            <div class="incidents-list">{ for rows }</div>
        </div>
    }
}
