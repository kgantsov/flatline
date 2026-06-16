mod charts;
mod detail;

use crate::api::{self, Incident, Monitor, MonitorCheck, MonitorNotification, NotificationChannel};
use crate::layout::Layout;
use crate::routes::Route;
use detail::MonitorDetail;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

// ── State types ───────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub(super) struct PageData {
    pub monitor: Monitor,
    pub checks: Vec<MonitorCheck>,
    pub incidents: Vec<Incident>,
    pub notifications: Vec<MonitorNotification>,
    pub channels: Vec<NotificationChannel>,
}

#[derive(Clone, PartialEq)]
pub(super) enum PageState {
    Loading,
    Loaded(Box<PageData>),
    Error(String),
}

#[derive(Clone, PartialEq)]
pub(super) enum Tab {
    Checks,
    Incidents,
    Notifications,
}

// ── Monitor page ──────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct MonitorPageProps {
    pub id: String,
}

#[function_component(MonitorPage)]
pub fn monitor_page(props: &MonitorPageProps) -> Html {
    let page_state = use_state(|| PageState::Loading);
    let active_tab = use_state(|| Tab::Checks);
    let delete_modal = use_state(|| false);
    let navigator = use_navigator().unwrap();

    let load = {
        let page_state = page_state.clone();
        let id = props.id.clone();
        move || {
            let page_state = page_state.clone();
            let id = id.clone();
            spawn_local(async move {
                let monitor = match api::fetch_monitor(&id).await {
                    Ok(m) => m,
                    Err(e) => { page_state.set(PageState::Error(e)); return; }
                };
                let checks = api::fetch_checks(&id, 100).await;
                let incidents = api::fetch_incidents(&id).await;
                let notifications = api::fetch_monitor_notifications(&id).await;
                let channels = api::fetch_all_channels().await;
                page_state.set(PageState::Loaded(Box::new(PageData {
                    monitor, checks, incidents, notifications, channels,
                })));
            });
        }
    };

    {
        let load = load.clone();
        let id = props.id.clone();
        use_effect_with(id, move |_| { load(); });
    }

    {
        let load = load.clone();
        use_effect_with((), move |_| {
            let interval = gloo_timers::callback::Interval::new(30_000, load);
            move || drop(interval)
        });
    }

    let on_reload = {
        let load = load.clone();
        Callback::from(move |_: ()| load())
    };

    let on_toggle = {
        let page_state = page_state.clone();
        let load = load.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let PageState::Loaded(data) = (*page_state).clone() {
                let enabled = !data.monitor.enabled;
                let id = data.monitor.id.to_string();
                let load = load.clone();
                spawn_local(async move {
                    let _ = api::toggle_monitor(&id, enabled).await;
                    load();
                });
            }
        })
    };

    let on_open_delete = {
        let delete_modal = delete_modal.clone();
        Callback::from(move |_: MouseEvent| delete_modal.set(true))
    };
    let on_close_delete = {
        let delete_modal = delete_modal.clone();
        Callback::from(move |_: MouseEvent| delete_modal.set(false))
    };
    let on_confirm_delete = {
        let page_state = page_state.clone();
        let delete_modal = delete_modal.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            if let PageState::Loaded(data) = (*page_state).clone() {
                let id = data.monitor.id.to_string();
                let delete_modal = delete_modal.clone();
                let navigator = navigator.clone();
                spawn_local(async move {
                    match api::delete_monitor(&id).await {
                        Ok(_) => navigator.push(&Route::Monitors),
                        Err(e) => {
                            web_sys::window()
                                .unwrap()
                                .alert_with_message(&format!("Failed to delete: {e}"))
                                .unwrap();
                            delete_modal.set(false);
                        }
                    }
                });
            }
        })
    };

    let on_tab_checks = {
        let active_tab = active_tab.clone();
        Callback::from(move |_: MouseEvent| active_tab.set(Tab::Checks))
    };
    let on_tab_incidents = {
        let active_tab = active_tab.clone();
        Callback::from(move |_: MouseEvent| active_tab.set(Tab::Incidents))
    };
    let on_tab_notifications = {
        let active_tab = active_tab.clone();
        Callback::from(move |_: MouseEvent| active_tab.set(Tab::Notifications))
    };

    let (toggle_label, toggle_hidden, edit_href, monitor_name) = match (*page_state).clone() {
        PageState::Loaded(ref data) => (
            if data.monitor.enabled { "Pause" } else { "Resume" },
            false,
            format!("/create?id={}", data.monitor.id),
            data.monitor.name.clone(),
        ),
        _ => ("Pause", true, "#".into(), "Monitor".into()),
    };

    let header_actions = html! {
        <>
            { if !toggle_hidden { html! {
                <>
                    <button class="btn btn-ghost" onclick={on_toggle}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <rect x="6" y="4" width="4" height="16"/>
                            <rect x="14" y="4" width="4" height="16"/>
                        </svg>
                        { toggle_label }
                    </button>
                    <a class="btn btn-ghost" href={edit_href}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
                            <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
                        </svg>
                        { "Edit" }
                    </a>
                </>
            }} else { html! {} }}
            <button class="btn btn-danger" onclick={on_open_delete.clone()}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                    stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/>
                    <path d="M10 11v6"/><path d="M14 11v6"/>
                </svg>
                { "Delete" }
            </button>
        </>
    };

    html! {
        <Layout header_actions={Some(header_actions)}>
            <main>
                { match (*page_state).clone() {
                    PageState::Loading => html! {
                        <div class="loading">
                            <div class="loading-spinner"></div>
                            { "Loading monitor…" }
                        </div>
                    },
                    PageState::Error(msg) => html! {
                        <>
                            <div class="breadcrumb">
                                <a href="/">{ "Monitors" }</a>
                                <span class="breadcrumb-sep">{ "/" }</span>
                                <span>{ "Error" }</span>
                            </div>
                            <div class="error-msg">{ format!("Failed to load monitor: {msg}") }</div>
                        </>
                    },
                    PageState::Loaded(data) => html! {
                        <MonitorDetail
                            data={(*data).clone()}
                            active_tab={(*active_tab).clone()}
                            on_tab_checks={on_tab_checks.clone()}
                            on_tab_incidents={on_tab_incidents.clone()}
                            on_tab_notifications={on_tab_notifications.clone()}
                            on_reload={on_reload.clone()}
                        />
                    },
                }}
            </main>

            { if *delete_modal { html! {
                <div class="modal-overlay"
                    onclick={let c = on_close_delete.clone();
                             Callback::from(move |e: MouseEvent| {
                                 if e.target() == e.current_target() { c.emit(e); }
                             })}>
                    <div class="modal">
                        <h3>{ "Delete monitor?" }</h3>
                        <p>
                            { format!("Delete \"{}\"? This will permanently remove all check history and incidents. This cannot be undone.", monitor_name) }
                        </p>
                        <div class="modal-actions">
                            <button class="btn btn-ghost" onclick={on_close_delete}>{ "Cancel" }</button>
                            <button class="btn btn-danger" onclick={on_confirm_delete}>{ "Delete monitor" }</button>
                        </div>
                    </div>
                </div>
            }} else { html! {} }}
        </Layout>
    }
}
