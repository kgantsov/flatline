use crate::api::{MonitorCheck, MonitorCheckStatus};
use crate::utils::{fmt_ms, fmt_time};
use yew::prelude::*;

/// Shared hover tooltip for bar-style charts (sparkline + response-time chart).
/// Shows time, latency, HTTP status code, and — for failed checks — the error.
pub fn bar_tooltip(c: &MonitorCheck) -> Html {
    let is_down = c.status == MonitorCheckStatus::Down;
    let code = c
        .status_code
        .map(|s| format!("HTTP {}", s))
        .unwrap_or_else(|| "no response".into());
    let code_cls = if is_down {
        "bar-tooltip-code down"
    } else {
        "bar-tooltip-code"
    };

    html! {
        <div class="bar-tooltip">
            <span class="bar-tooltip-value">{ fmt_ms(c.response_time_ms) }</span>
            <span class="bar-tooltip-time">{ fmt_time(&c.checked_at.to_rfc3339()) }</span>
            <span class={code_cls}>{ code }</span>
            { match &c.error_message {
                Some(err) if !err.is_empty() => html! {
                    <span class="bar-tooltip-error">{ err }</span>
                },
                _ => html! {},
            }}
        </div>
    }
}
