use crate::api::{self, PublicStatusPage as PublicStatusPageData};
use gloo_timers::callback::Interval;
use js_sys::Date;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub slug: String,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Loaded(PublicStatusPageData),
    Error(String),
}

/// 0 min → green (operational). Any downtime → yellow→orange→red.
/// u32::MAX → grey (no data).
fn day_color(minutes: u32) -> String {
    if minutes == u32::MAX {
        return "hsl(220,10%,30%)".into();
    }
    if minutes == 0 {
        return "#22c55e".into(); // --up
    }
    // 1 min = yellow  ·  30 min = orange  ·  ≥60 min = red (#ef4444)
    let t = (minutes.min(60) as f64) / 60.0;
    let h = 55.0 * (1.0 - t);           // 55 → 0
    let s = 92.0 - 8.0 * t;             // 92 → 84
    let l = 50.0 + 10.0 * t;            // 50 → 60
    format!("hsl({h:.0},{s:.0}%,{l:.0}%)")
}

fn downtime_label(mins: u32) -> String {
    if mins == u32::MAX { return "No data".into(); }
    if mins == 0        { return "No downtime".into(); }
    if mins >= 60       { return format!("{}h {}m downtime", mins / 60, mins % 60); }
    format!("{mins} min downtime")
}

fn sparkline(downtime: &[u32]) -> Html {
    let placeholder;
    let data: &[u32] = if downtime.is_empty() {
        placeholder = vec![u32::MAX; 90];
        &placeholder
    } else {
        downtime
    };

    let n = data.len();
    let months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let now_ms = Date::now();

    let bars: Html = data.iter().enumerate().map(|(i, &mins)| {
        let days_ago = n - 1 - i;
        let color = day_color(mins);
        let d = Date::new(&wasm_bindgen::JsValue::from_f64(
            now_ms - days_ago as f64 * 86_400_000.0,
        ));
        let date_str = format!("{} {}", months[d.get_month() as usize], d.get_date());
        let dt_str   = downtime_label(mins);

        html! {
            <div class="bar-col sp-bar-col" key={i}>
                <div class="sp-day-bar" style={format!("background:{color}")}></div>
                <div class="bar-tooltip">
                    <span class="bar-tooltip-value">{ date_str }</span>
                    <span class="bar-tooltip-time">{ dt_str }</span>
                </div>
            </div>
        }
    }).collect();

    html! {
        <div class="sp-bars">{ bars }</div>
    }
}

#[function_component(PublicStatusPageComponent)]
pub fn public_status_page(props: &Props) -> Html {
    let state = use_state(|| LoadState::Loading);

    // Initial fetch.
    {
        let state = state.clone();
        let slug = props.slug.clone();
        use_effect_with(slug.clone(), move |_| {
            spawn_local(async move {
                match api::fetch_public_status_page(&slug).await {
                    Ok(data) => state.set(LoadState::Loaded(data)),
                    Err(e) => state.set(LoadState::Error(e)),
                }
            });
        });
    }

    // Auto-refresh interval — driven by the page's own refresh_interval setting.
    // We extract the interval from current state; if not yet loaded we skip setting
    // up the timer until the first successful load re-renders with the real value.
    let refresh_secs = match &*state {
        LoadState::Loaded(data) => data.page.refresh_interval,
        _ => 0,
    };
    {
        let state = state.clone();
        let slug = props.slug.clone();
        use_effect_with(refresh_secs, move |&secs| {
            let interval = (secs > 0).then(|| {
                Interval::new(secs * 1000, move || {
                    let state = state.clone();
                    let slug = slug.clone();
                    spawn_local(async move {
                        if let Ok(data) = api::fetch_public_status_page(&slug).await {
                            state.set(LoadState::Loaded(data));
                        }
                    });
                })
            });
            move || drop(interval)
        });
    }

    let body = match (*state).clone() {
        LoadState::Loading => html! {
            <div class="sp-center">
                <div class="loading-spinner"></div>
            </div>
        },
        LoadState::Error(e) => html! {
            <div class="sp-center">
                <div class="sp-error-title">{ "Page not found" }</div>
                <div class="sp-error-sub">{ e }</div>
            </div>
        },
        LoadState::Loaded(data) => {
            let all_up = data.monitors.iter().all(|m| m.is_up);
            let any_up = data.monitors.iter().any(|m| m.is_up);
            let (badge_cls, badge_label) = if all_up {
                ("sp-overall sp-overall-up", "All Systems Operational")
            } else if any_up {
                ("sp-overall sp-overall-degraded", "Partial Outage")
            } else {
                ("sp-overall sp-overall-down", "Major Outage")
            };
            let refresh = data.page.refresh_interval;

            html! {
                <>
                    // Use <div> not <header> — the global header{} CSS would override it.
                    <div class="sp-header">
                        <div class="sp-header-inner">
                            <h1 class="sp-title">{ &data.page.name }</h1>
                            { if let Some(d) = &data.page.description {
                                html! { <p class="sp-desc">{ d }</p> }
                            } else { html! {} }}
                            <div class={badge_cls}>
                                <span class="sp-overall-dot"></span>
                                { badge_label }
                            </div>
                        </div>
                    </div>

                    // Use <div> not <main> — global main{} sets max-width:1100px.
                    <div class="sp-main">
                        { for data.monitors.iter().map(|entry| {
                            let is_up = entry.is_up;
                            let uptime = entry.stats.as_ref()
                                .map_or(100.0, |s| s.uptime_90d * 100.0);
                            let p50 = entry.stats.as_ref().map_or(0, |s| s.p50_90d);
                            let (dot_cls, status_label) = if is_up {
                                ("sp-dot sp-dot-up", "Operational")
                            } else {
                                ("sp-dot sp-dot-down", "Outage")
                            };
                            html! {
                                <div class="sp-monitor" key={entry.monitor.id.to_string()}>
                                    <div class="sp-monitor-header">
                                        <span class="sp-monitor-name">
                                            { &entry.monitor.name }
                                        </span>
                                        <span class="sp-monitor-status">
                                            <span class={dot_cls}></span>
                                            { status_label }
                                        </span>
                                    </div>
                                    <div class="sp-sparkline">
                                        { sparkline(&entry.day_downtime_minutes) }
                                    </div>
                                    <div class="sp-monitor-footer">
                                        <span>{ "90 days ago" }</span>
                                        <span class="sp-monitor-meta">
                                            { format!("Uptime 90d: {uptime:.2}%  ·  P50: {p50}ms") }
                                        </span>
                                        <span>{ "Today" }</span>
                                    </div>
                                </div>
                            }
                        })}
                    </div>

                    // Use <div> not <footer> — global footer{} adds border-top+padding.
                    <div class="sp-footer">
                        { format!("Auto-refreshes every {refresh}s  ·  Powered by ") }
                        <a href="/">{ "Flatline" }</a>
                    </div>
                </>
            }
        }
    };

    html! {
        <div class="sp-page">
            { body }
        </div>
    }
}
