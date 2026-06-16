mod dashboard;

use crate::api::{self, Incident, Monitor, MonitorCheck};
use crate::components::StatsBar;
use crate::layout::{Layout, NavActive};
use dashboard::Dashboard;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

// ── Dashboard data ─────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub(super) struct DashboardData {
    pub monitors: Vec<Monitor>,
    pub checks: Vec<Vec<MonitorCheck>>,
    pub incidents: Vec<Vec<Incident>>,
}

#[derive(Clone, PartialEq)]
pub(super) enum LoadState {
    Loading,
    Loaded(DashboardData),
    Error(String),
}

// ── Monitors page ─────────────────────────────────────────────────────────────

#[function_component(MonitorsPage)]
pub fn monitors_page() -> Html {
    let state = use_state(|| LoadState::Loading);
    let last_updated = use_state(|| String::from("Loading…"));

    let load = {
        let state = state.clone();
        let last_updated = last_updated.clone();
        move || {
            let state = state.clone();
            let last_updated = last_updated.clone();
            spawn_local(async move {
                state.set(LoadState::Loading);
                last_updated.set("Refreshing…".into());

                let monitors = match api::fetch_monitors().await {
                    Ok(m) => m,
                    Err(e) => {
                        state.set(LoadState::Error(format!("Failed to load monitors: {e}")));
                        last_updated.set("Error loading data".into());
                        return;
                    }
                };

                let (checks_all, incidents_all) = {
                    let ids: Vec<String> = monitors.iter().map(|m| m.id.to_string()).collect();
                    let mut checks_futs = Vec::new();
                    let mut incidents_futs = Vec::new();
                    for id in &ids {
                        checks_futs.push(api::fetch_checks(id, 30));
                        incidents_futs.push(api::fetch_incidents(id));
                    }
                    let mut checks_all = Vec::new();
                    for f in checks_futs {
                        checks_all.push(f.await);
                    }
                    let mut incidents_all = Vec::new();
                    for f in incidents_futs {
                        incidents_all.push(f.await);
                    }
                    (checks_all, incidents_all)
                };

                let count = monitors.len();
                state.set(LoadState::Loaded(DashboardData {
                    monitors,
                    checks: checks_all,
                    incidents: incidents_all,
                }));
                last_updated.set(format!(
                    "Last updated just now · {} monitor{}",
                    count,
                    if count != 1 { "s" } else { "" }
                ));
            });
        }
    };

    {
        let load = load.clone();
        use_effect_with((), move |_| {
            load();
        });
    }

    {
        let load = load.clone();
        use_effect_with((), move |_| {
            let interval = gloo_timers::callback::Interval::new(30_000, move || {
                load();
            });
            move || drop(interval)
        });
    }

    let on_refresh = {
        let load = load.clone();
        Callback::from(move |_: MouseEvent| load())
    };

    let header_actions = html! {
        <a href="/create" class="btn btn-primary">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                stroke-width="2.5" stroke-linecap="round">
                <line x1="12" y1="5" x2="12" y2="19" />
                <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            { "Add monitor" }
        </a>
    };

    html! {
        <Layout active={NavActive::Monitors} header_actions={Some(header_actions)}>
            <main>
                <div class="page-header">
                    <div>
                        <h1>{ "Monitors" }</h1>
                        <p>{ (*last_updated).clone() }</p>
                    </div>
                    <button class="btn btn-ghost" onclick={on_refresh}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M23 4v6h-6" />
                            <path d="M1 20v-6h6" />
                            <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15" />
                        </svg>
                        { "Refresh" }
                    </button>
                </div>

                { match (*state).clone() {
                    LoadState::Loading => html! {
                        <>
                            <StatsBar total={"—"} up={"—"} down={"—"} incidents={"—"} />
                            <div class="loading">
                                <div class="loading-spinner"></div>
                                { "Loading monitors…" }
                            </div>
                        </>
                    },
                    LoadState::Error(msg) => html! {
                        <div class="error-msg">{ msg }</div>
                    },
                    LoadState::Loaded(data) => html! { <Dashboard data={data} /> },
                }}
            </main>
        </Layout>
    }
}
