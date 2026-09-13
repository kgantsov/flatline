use crate::api::{self, Monitor, StatusPage, StatusPageMonitor};
use crate::layout::{Layout, NavActive};
use crate::utils::fmt_date;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[derive(Clone, PartialEq)]
struct PageData {
    page: StatusPage,
    monitors_on_page: Vec<StatusPageMonitor>,
    all_monitors: Vec<Monitor>,
}

#[derive(Clone, PartialEq)]
enum LoadState {
    Loading,
    Loaded(PageData),
    Error(String),
}

#[function_component(StatusPageDetailPage)]
pub fn status_page_detail(props: &Props) -> Html {
    let state = use_state(|| LoadState::Loading);
    let remove_target: UseStateHandle<Option<StatusPageMonitor>> = use_state(|| None);

    let load = {
        let state = state.clone();
        let id = props.id.clone();
        Callback::from(move |_: ()| {
            let state = state.clone();
            let id = id.clone();
            spawn_local(async move {
                let page = match api::fetch_status_page(&id).await {
                    Ok(p) => p,
                    Err(e) => { state.set(LoadState::Error(e)); return; }
                };
                let monitors_on_page = api::fetch_page_monitors(&id).await;
                let all_monitors = api::fetch_monitors()
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|s| s.monitor)
                    .collect();
                state.set(LoadState::Loaded(PageData { page, monitors_on_page, all_monitors }));
            });
        })
    };

    {
        let load = load.clone();
        use_effect_with(props.id.clone(), move |_| load.emit(()));
    }

    let open_remove = {
        let remove_target = remove_target.clone();
        Callback::from(move |link: StatusPageMonitor| remove_target.set(Some(link)))
    };

    let close_remove = {
        let remove_target = remove_target.clone();
        Callback::from(move |_: MouseEvent| remove_target.set(None))
    };

    let confirm_remove = {
        let remove_target = remove_target.clone();
        let load = load.clone();
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(link) = (*remove_target).clone() else { return; };
            let page_id = link.status_page_id.to_string();
            let monitor_id = link.monitor_id.to_string();
            let remove_target = remove_target.clone();
            let load = load.clone();
            let state = state.clone();
            spawn_local(async move {
                state.set(LoadState::Loading);
                let _ = api::remove_monitor_from_page(&page_id, &monitor_id).await;
                remove_target.set(None);
                load.emit(());
            });
        })
    };

    let on_add = {
        let load = load.clone();
        let state = state.clone();
        let id = props.id.clone();
        Callback::from(move |monitor_id: String| {
            let load = load.clone();
            let state = state.clone();
            let id = id.clone();
            spawn_local(async move {
                state.set(LoadState::Loading);
                let _ = api::add_monitor_to_page(&id, &monitor_id).await;
                load.emit(());
            });
        })
    };

    let body = match (*state).clone() {
        LoadState::Loading => html! {
            <div class="loading">
                <div class="loading-spinner"></div>
                { "Loading\u{2026}" }
            </div>
        },
        LoadState::Error(e) => html! {
            <div class="error-msg">{ format!("Failed to load: {e}") }</div>
        },
        LoadState::Loaded(data) => {
            let linked_ids: std::collections::HashSet<String> = data.monitors_on_page
                .iter().map(|m| m.monitor_id.to_string()).collect();
            let slug = data.page.slug.clone();
            let page_id = data.page.id.to_string();
            let added = fmt_date(&data.page.created_at.to_rfc3339());

            let linked_monitors: Vec<(&StatusPageMonitor, Option<&Monitor>)> = data.monitors_on_page
                .iter()
                .map(|link| {
                    let m = data.all_monitors.iter().find(|m| m.id == link.monitor_id);
                    (link, m)
                })
                .collect();

            let available: Vec<&Monitor> = data.all_monitors.iter()
                .filter(|m| !linked_ids.contains(&m.id.to_string()))
                .collect();

            html! {
                <>
                    <div class="breadcrumb">
                        <a href="/status-pages">{ "Status Pages" }</a>
                        <span class="breadcrumb-sep">{ "/" }</span>
                        <span>{ &data.page.name }</span>
                    </div>

                    <div style="display:flex;align-items:flex-start;justify-content:space-between;margin-bottom:24px;gap:16px">
                        <div>
                            <div class="page-title" style="margin-bottom:4px">{ &data.page.name }</div>
                            { if let Some(d) = &data.page.description {
                                html! { <div class="page-desc" style="margin-bottom:4px">{ d }</div> }
                            } else { html! {} }}
                            <div class="channel-meta">
                                { format!("Public URL: /s/{slug}  ·  refresh every {}s  ·  created {added}", data.page.refresh_interval) }
                            </div>
                        </div>
                        <div style="display:flex;gap:8px;flex-shrink:0">
                            <a href={format!("/s/{slug}")} target="_blank" class="btn btn-ghost btn-sm">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                    stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/>
                                    <polyline points="15 3 21 3 21 9"/>
                                    <line x1="10" y1="14" x2="21" y2="3"/>
                                </svg>
                                { "View public page" }
                            </a>
                            <a href={format!("/status-pages/{page_id}/edit")} class="btn btn-ghost btn-sm">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                    stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
                                    <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
                                </svg>
                                { "Edit settings" }
                            </a>
                        </div>
                    </div>

                    // ── Monitors on this page ──────────────────────────────────────

                    <div class="section-title" style="margin-bottom:14px">
                        { format!("Monitors on this page ({})", linked_monitors.len()) }
                    </div>

                    { if linked_monitors.is_empty() { html! {
                        <div class="empty-state">
                            <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none"
                                stroke="currentColor" stroke-width="1.5"
                                stroke-linecap="round" stroke-linejoin="round">
                                <polyline points="3 12 6 12 9 4 12 20 15 12 18 12 21 12"/>
                            </svg>
                            <h3>{ "No monitors yet" }</h3>
                            <p>{ "Add monitors below to display them on this status page." }</p>
                        </div>
                    }} else { html! {
                        <div class="channels-list" style="margin-bottom:32px">
                            { for linked_monitors.iter().map(|(link, monitor)| {
                                let link = (*link).clone();
                                let open_remove = open_remove.clone();
                                let name = monitor
                                    .map(|m| m.name.clone())
                                    .unwrap_or_else(|| link.monitor_id.to_string());
                                let added = fmt_date(&link.created_at.to_rfc3339());
                                html! {
                                    <div class="channel-card" key={link.monitor_id.to_string()}>
                                        <div class="channel-info">
                                            <div class="channel-name">{ name }</div>
                                            <div class="channel-meta">{ format!("Added {added}") }</div>
                                        </div>
                                        <div class="channel-actions">
                                            <button class="btn btn-danger btn-sm"
                                                onclick={Callback::from(move |_: MouseEvent| open_remove.emit(link.clone()))}>
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                                    stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                    <polyline points="3 6 5 6 21 6"/>
                                                    <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/>
                                                </svg>
                                                { "Remove" }
                                            </button>
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    }}}

                    // ── Add monitors ───────────────────────────────────────────────

                    { if !available.is_empty() { html! {
                        <>
                            <div class="section-title" style="margin-bottom:14px">{ "Add monitors" }</div>
                            <div class="channels-list">
                                { for available.iter().map(|m| {
                                    let mid = m.id.to_string();
                                    let on_add = on_add.clone();
                                    let add_id = mid.clone();
                                    html! {
                                        <div class="channel-card" key={mid.clone()}>
                                            <div class="channel-info">
                                                <div class="channel-name">{ &m.name }</div>
                                            </div>
                                            <div class="channel-actions">
                                                <button class="btn btn-ghost btn-sm"
                                                    onclick={Callback::from(move |_: MouseEvent| on_add.emit(add_id.clone()))}>
                                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                                        stroke-width="2.5" stroke-linecap="round">
                                                        <line x1="12" y1="5" x2="12" y2="19"/>
                                                        <line x1="5" y1="12" x2="19" y2="12"/>
                                                    </svg>
                                                    { "Add" }
                                                </button>
                                            </div>
                                        </div>
                                    }
                                })}
                            </div>
                        </>
                    }} else { html! {} }}
                </>
            }
        }
    };

    let remove_name = remove_target.as_ref().and_then(|link| {
        if let LoadState::Loaded(ref data) = *state {
            data.all_monitors.iter()
                .find(|m| m.id == link.monitor_id)
                .map(|m| m.name.clone())
                .or_else(|| Some(link.monitor_id.to_string()))
        } else {
            None
        }
    });

    html! {
        <Layout active={NavActive::StatusPages}>
            <main>
                { body }
            </main>

            { if let Some(name) = remove_name { html! {
                <div class="modal-overlay" onclick={close_remove.clone()}>
                    <div class="modal"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                        <h3>{ "Remove monitor?" }</h3>
                        <p>{ format!("Remove \"{}\" from this status page? It will no longer appear publicly.", name) }</p>
                        <div class="modal-actions">
                            <button class="btn btn-ghost" onclick={close_remove}>{ "Cancel" }</button>
                            <button class="btn btn-danger" onclick={confirm_remove}>{ "Remove" }</button>
                        </div>
                    </div>
                </div>
            }} else { html! {} }}
        </Layout>
    }
}
