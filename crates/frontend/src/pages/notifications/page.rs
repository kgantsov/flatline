use super::{
    channel_modal::ChannelModal, channel_type_key, channel_type_label, channel_url,
};
use crate::api::{self, NotificationChannel};
use crate::layout::{Layout, NavActive};
use crate::utils::fmt_date;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum ModalState {
    Closed,
    Create,
    Edit(NotificationChannel),
}

#[function_component(NotificationsPage)]
pub fn notifications_page() -> Html {
    let channels: UseStateHandle<Option<Result<Vec<NotificationChannel>, String>>> =
        use_state(|| None);
    let modal = use_state(|| ModalState::Closed);
    let delete_target: UseStateHandle<Option<NotificationChannel>> = use_state(|| None);

    // ── Load ───────────────────────────────────────────────────────────────────

    let reload = {
        let channels = channels.clone();
        Callback::from(move |_: ()| {
            let channels = channels.clone();
            spawn_local(async move {
                channels.set(Some(api::fetch_notification_channels().await));
            });
        })
    };

    {
        let reload = reload.clone();
        use_effect_with((), move |_| reload.emit(()));
    }

    // ── Modal open/close ───────────────────────────────────────────────────────

    let open_create = {
        let modal = modal.clone();
        Callback::from(move |_: MouseEvent| modal.set(ModalState::Create))
    };

    let open_edit = {
        let modal = modal.clone();
        Callback::from(move |ch: NotificationChannel| modal.set(ModalState::Edit(ch)))
    };

    let close_modal = {
        let modal = modal.clone();
        Callback::from(move |_: ()| modal.set(ModalState::Closed))
    };

    let on_saved = {
        let modal = modal.clone();
        let reload = reload.clone();
        Callback::from(move |_: ()| {
            modal.set(ModalState::Closed);
            reload.emit(());
        })
    };

    // ── Delete ─────────────────────────────────────────────────────────────────

    let open_delete = {
        let delete_target = delete_target.clone();
        Callback::from(move |ch: NotificationChannel| delete_target.set(Some(ch)))
    };

    let close_delete = {
        let delete_target = delete_target.clone();
        Callback::from(move |_: MouseEvent| delete_target.set(None))
    };

    let confirm_delete = {
        let delete_target = delete_target.clone();
        let reload = reload.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(ch) = (*delete_target).clone() else {
                return;
            };
            let delete_target = delete_target.clone();
            let reload = reload.clone();
            spawn_local(async move {
                if api::delete_channel(&ch.id.to_string()).await.is_ok() {
                    delete_target.set(None);
                    reload.emit(());
                }
            });
        })
    };

    // ── Header ─────────────────────────────────────────────────────────────────

    let header_actions = {
        let open_create = open_create.clone();
        html! {
            <button class="btn btn-primary" onclick={open_create}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                    stroke-width="2.5" stroke-linecap="round">
                    <line x1="12" y1="5" x2="12" y2="19"/>
                    <line x1="5" y1="12" x2="19" y2="12"/>
                </svg>
                { "New channel" }
            </button>
        }
    };

    // ── Channel list ───────────────────────────────────────────────────────────

    let channels_content = match &*channels {
        None => html! {
            <div class="loading">
                <div class="loading-spinner"></div>
                { "Loading channels\u{2026}" }
            </div>
        },
        Some(Err(e)) => html! {
            <div class="error-msg">{ format!("Failed to load channels: {e}") }</div>
        },
        Some(Ok(chs)) if chs.is_empty() => {
            let open_create = open_create.clone();
            html! {
                <div class="empty-state">
                    <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none"
                        stroke="currentColor" stroke-width="1.5"
                        stroke-linecap="round" stroke-linejoin="round">
                        <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/>
                        <path d="M13.73 21a2 2 0 01-3.46 0"/>
                    </svg>
                    <h3>{ "No notification channels" }</h3>
                    <p>{ "Add a channel to receive alerts when monitors go down." }</p>
                    <button class="btn btn-primary" onclick={open_create}>{ "New channel" }</button>
                </div>
            }
        }
        Some(Ok(chs)) => {
            let open_edit = open_edit.clone();
            let open_delete = open_delete.clone();
            html! {
                <div class="channels-list">
                    { for chs.iter().map(|ch| {
                        let ch_edit = ch.clone();
                        let ch_del = ch.clone();
                        let open_edit = open_edit.clone();
                        let open_delete = open_delete.clone();
                        let type_key = channel_type_key(&ch.config);
                        let type_label = channel_type_label(&ch.config);
                        let url = channel_url(&ch.config).to_string();
                        let added = fmt_date(&ch.created_at.to_rfc3339());
                        html! {
                            <div class="channel-card">
                                <div class="channel-info">
                                    <div class="channel-name">
                                        <span class={format!("type-badge {type_key}")}>
                                            { type_label }
                                        </span>
                                        { &ch.name }
                                    </div>
                                    <div class="channel-url">{ url }</div>
                                    <div class="channel-meta">{ format!("Added {added}") }</div>
                                </div>
                                <div class="channel-actions">
                                    <button class="btn btn-ghost btn-sm"
                                        onclick={Callback::from(move |_: MouseEvent| open_edit.emit(ch_edit.clone()))}>
                                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/>
                                            <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/>
                                        </svg>
                                        { "Edit" }
                                    </button>
                                    <button class="btn btn-danger btn-sm"
                                        onclick={Callback::from(move |_: MouseEvent| open_delete.emit(ch_del.clone()))}>
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

    // ── Render ─────────────────────────────────────────────────────────────────

    let editing_channel = match &*modal {
        ModalState::Edit(ch) => Some(ch.clone()),
        _ => None,
    };
    let modal_open = *modal != ModalState::Closed;

    html! {
        <Layout active={NavActive::Notifications} header_actions={header_actions}>
            <main>
                <div class="page-header">
                    <div>
                        <h1>{ "Notification channels" }</h1>
                        <p>{ "Configure where to send alerts when monitors go down or recover." }</p>
                    </div>
                </div>
                <div class="section-title" style="margin-bottom:14px">{ "Channels" }</div>
                { channels_content }
            </main>

            { if modal_open { html! {
                <ChannelModal
                    channel={editing_channel}
                    on_close={close_modal}
                    on_saved={on_saved}
                />
            }} else { html! {} }}

            { if let Some(ch) = &*delete_target { html! {
                <div class="modal-overlay" onclick={close_delete.clone()}>
                    <div class="modal"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                        <h3>{ "Delete channel?" }</h3>
                        <p>
                            { format!(
                                "Delete \"{}\"? Any monitors using it will stop receiving \
                                 alerts. This cannot be undone.",
                                ch.name
                            )}
                        </p>
                        <div class="modal-actions">
                            <button class="btn btn-ghost" onclick={close_delete.clone()}>
                                { "Cancel" }
                            </button>
                            <button class="btn btn-danger" onclick={confirm_delete.clone()}>
                                { "Delete channel" }
                            </button>
                        </div>
                    </div>
                </div>
            }} else { html! {} }}
        </Layout>
    }
}
