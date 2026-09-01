use crate::api::{self, StatusPage};
use crate::layout::{Layout, NavActive};
use crate::utils::fmt_date;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(StatusPagesPage)]
pub fn status_pages_page() -> Html {
    let pages: UseStateHandle<Option<Result<Vec<StatusPage>, String>>> = use_state(|| None);
    let delete_target: UseStateHandle<Option<StatusPage>> = use_state(|| None);

    let reload = {
        let pages = pages.clone();
        Callback::from(move |_: ()| {
            let pages = pages.clone();
            spawn_local(async move {
                pages.set(Some(api::fetch_status_pages().await));
            });
        })
    };

    {
        let reload = reload.clone();
        use_effect_with((), move |_| reload.emit(()));
    }

    let open_delete = {
        let delete_target = delete_target.clone();
        Callback::from(move |page: StatusPage| delete_target.set(Some(page)))
    };

    let close_delete = {
        let delete_target = delete_target.clone();
        Callback::from(move |_: MouseEvent| delete_target.set(None))
    };

    let confirm_delete = {
        let delete_target = delete_target.clone();
        let reload = reload.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(page) = (*delete_target).clone() else { return; };
            let delete_target = delete_target.clone();
            let reload = reload.clone();
            spawn_local(async move {
                if api::delete_status_page(&page.id.to_string()).await.is_ok() {
                    delete_target.set(None);
                    reload.emit(());
                }
            });
        })
    };

    let header_actions = html! {
        <a href="/status-pages/create" class="btn btn-primary">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                stroke-width="2.5" stroke-linecap="round">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            { "New status page" }
        </a>
    };

    let content = match &*pages {
        None => html! {
            <div class="loading">
                <div class="loading-spinner"></div>
                { "Loading\u{2026}" }
            </div>
        },
        Some(Err(e)) => html! {
            <div class="error-msg">{ format!("Failed to load status pages: {e}") }</div>
        },
        Some(Ok(list)) if list.is_empty() => html! {
            <div class="empty-state">
                <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" stroke-width="1.5"
                    stroke-linecap="round" stroke-linejoin="round">
                    <rect x="3" y="3" width="18" height="18" rx="2"/>
                    <line x1="3" y1="9" x2="21" y2="9"/>
                    <line x1="9" y1="21" x2="9" y2="9"/>
                </svg>
                <h3>{ "No status pages" }</h3>
                <p>{ "Create a status page to share monitor status with your users." }</p>
                <a href="/status-pages/create" class="btn btn-primary">{ "New status page" }</a>
            </div>
        },
        Some(Ok(list)) => {
            let open_delete = open_delete.clone();
            html! {
                <div class="channels-list">
                    { for list.iter().map(|page| {
                        let page_del = page.clone();
                        let open_delete = open_delete.clone();
                        let id = page.id.to_string();
                        let slug = page.slug.clone();
                        let added = fmt_date(&page.created_at.to_rfc3339());
                        let public_url = format!("/s/{slug}");
                        html! {
                            <div class="channel-card" key={id.clone()}>
                                <div class="channel-info">
                                    <div class="channel-name">
                                        <span class="type-badge webhook">{ "Page" }</span>
                                        { &page.name }
                                    </div>
                                    { if let Some(desc) = &page.description {
                                        html! { <div class="channel-url">{ desc }</div> }
                                    } else { html! {} }}
                                    <div class="channel-meta">
                                        { format!("/{slug} · refresh every {}s · added {added}", page.refresh_interval) }
                                    </div>
                                </div>
                                <div class="channel-actions">
                                    <a href={public_url} target="_blank" class="btn btn-ghost btn-sm">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/>
                                            <polyline points="15 3 21 3 21 9"/>
                                            <line x1="10" y1="14" x2="21" y2="3"/>
                                        </svg>
                                        { "View" }
                                    </a>
                                    <a href={format!("/status-pages/{id}")} class="btn btn-ghost btn-sm">
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
                                            <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
                                        </svg>
                                        { "Edit" }
                                    </a>
                                    <button class="btn btn-danger btn-sm"
                                        onclick={Callback::from(move |_: MouseEvent| open_delete.emit(page_del.clone()))}>
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <polyline points="3 6 5 6 21 6"/>
                                            <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/>
                                        </svg>
                                        { "Delete" }
                                    </button>
                                </div>
                            </div>
                        }
                    })}
                </div>
            }
        }
    };

    html! {
        <Layout active={NavActive::StatusPages} header_actions={Some(header_actions)}>
            <main>
                <div class="page-header">
                    <div>
                        <h1>{ "Status Pages" }</h1>
                        <p>{ "Share the status of your monitors with your users." }</p>
                    </div>
                </div>
                <div class="section-title" style="margin-bottom:14px">{ "Pages" }</div>
                { content }
            </main>

            { if let Some(page) = &*delete_target { html! {
                <div class="modal-overlay" onclick={close_delete.clone()}>
                    <div class="modal"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                        <h3>{ "Delete status page?" }</h3>
                        <p>
                            { format!(
                                "Delete \"{}\"? The public URL /s/{} will stop working. \
                                 This cannot be undone.",
                                page.name, page.slug
                            )}
                        </p>
                        <div class="modal-actions">
                            <button class="btn btn-ghost" onclick={close_delete.clone()}>
                                { "Cancel" }
                            </button>
                            <button class="btn btn-danger" onclick={confirm_delete.clone()}>
                                { "Delete page" }
                            </button>
                        </div>
                    </div>
                </div>
            }} else { html! {} }}
        </Layout>
    }
}
