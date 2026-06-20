use crate::api::{
    self, NotificationChannel, NotificationChannelConfig, NotificationChannelConfigInput,
    NotificationChannelFormData,
};
use crate::layout::{Layout, NavActive};
use crate::utils::fmt_date;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

fn is_valid_url(s: &str) -> bool {
    let s = s.trim();
    (s.starts_with("http://") || s.starts_with("https://")) && s.len() > 8
}

fn channel_url(config: &NotificationChannelConfig) -> &str {
    match config {
        NotificationChannelConfig::Webhook { url } => url.as_str(),
        NotificationChannelConfig::Slack { webhook_url } => webhook_url.as_str(),
        NotificationChannelConfig::Telegram { url, .. } => url.as_str(),
    }
}

fn channel_type_key(config: &NotificationChannelConfig) -> &'static str {
    match config {
        NotificationChannelConfig::Webhook { .. } => "webhook",
        NotificationChannelConfig::Slack { .. } => "slack",
        NotificationChannelConfig::Telegram { .. } => "telegram",
    }
}

fn channel_type_label(config: &NotificationChannelConfig) -> &'static str {
    match config {
        NotificationChannelConfig::Webhook { .. } => "Webhook",
        NotificationChannelConfig::Slack { .. } => "Slack",
        NotificationChannelConfig::Telegram { .. } => "Telegram",
    }
}

#[derive(Clone, PartialEq)]
enum ChannelModal {
    Closed,
    Create,
    Edit(NotificationChannel),
}

#[function_component(NotificationsPage)]
pub fn notifications_page() -> Html {
    let channels: UseStateHandle<Option<Result<Vec<NotificationChannel>, String>>> =
        use_state(|| None);
    let modal = use_state(|| ChannelModal::Closed);
    let delete_target: UseStateHandle<Option<NotificationChannel>> = use_state(|| None);

    // Form state
    let form_name = use_state(String::new);
    let form_type = use_state(|| "webhook".to_string());
    let form_webhook_url = use_state(String::new);
    let form_slack_url = use_state(String::new);
    let form_telegram_url = use_state(String::new);
    let form_telegram_chat_id = use_state(String::new);
    let name_err = use_state(|| false);
    let url_err = use_state(|| false);
    let chat_id_err = use_state(|| false);
    let submitting = use_state(|| false);
    let modal_alert: UseStateHandle<Option<String>> = use_state(|| None);

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

    // ── Open create modal ──────────────────────────────────────────────────────

    let open_create = {
        let modal = modal.clone();
        let form_name = form_name.clone();
        let form_type = form_type.clone();
        let form_webhook_url = form_webhook_url.clone();
        let form_slack_url = form_slack_url.clone();
        let form_telegram_url = form_telegram_url.clone();
        let form_telegram_chat_id = form_telegram_chat_id.clone();
        let name_err = name_err.clone();
        let url_err = url_err.clone();
        let chat_id_err = chat_id_err.clone();
        let submitting = submitting.clone();
        let modal_alert = modal_alert.clone();
        Callback::from(move |_: MouseEvent| {
            form_name.set(String::new());
            form_type.set("webhook".to_string());
            form_webhook_url.set(String::new());
            form_slack_url.set(String::new());
            form_telegram_url.set(String::new());
            form_telegram_chat_id.set(String::new());
            name_err.set(false);
            url_err.set(false);
            chat_id_err.set(false);
            submitting.set(false);
            modal_alert.set(None);
            modal.set(ChannelModal::Create);
        })
    };

    // ── Open edit modal ────────────────────────────────────────────────────────

    let open_edit = {
        let modal = modal.clone();
        let form_name = form_name.clone();
        let form_type = form_type.clone();
        let form_webhook_url = form_webhook_url.clone();
        let form_slack_url = form_slack_url.clone();
        let form_telegram_url = form_telegram_url.clone();
        let form_telegram_chat_id = form_telegram_chat_id.clone();
        let name_err = name_err.clone();
        let url_err = url_err.clone();
        let chat_id_err = chat_id_err.clone();
        let submitting = submitting.clone();
        let modal_alert = modal_alert.clone();
        Callback::from(move |ch: NotificationChannel| {
            form_name.set(ch.name.clone());
            match &ch.config {
                NotificationChannelConfig::Webhook { url } => {
                    form_type.set("webhook".to_string());
                    form_webhook_url.set(url.clone());
                    form_slack_url.set(String::new());
                    form_telegram_url.set(String::new());
                    form_telegram_chat_id.set(String::new());
                }
                NotificationChannelConfig::Slack { webhook_url } => {
                    form_type.set("slack".to_string());
                    form_webhook_url.set(String::new());
                    form_slack_url.set(webhook_url.clone());
                    form_telegram_url.set(String::new());
                    form_telegram_chat_id.set(String::new());
                }
                NotificationChannelConfig::Telegram { url, chat_id } => {
                    form_type.set("telegram".to_string());
                    form_webhook_url.set(String::new());
                    form_slack_url.set(String::new());
                    form_telegram_url.set(url.clone());
                    form_telegram_chat_id.set(chat_id.clone());
                }
            }
            name_err.set(false);
            url_err.set(false);
            chat_id_err.set(false);
            submitting.set(false);
            modal_alert.set(None);
            modal.set(ChannelModal::Edit(ch));
        })
    };

    // ── Close modal ────────────────────────────────────────────────────────────

    let close_modal = {
        let modal = modal.clone();
        Callback::from(move |_: MouseEvent| modal.set(ChannelModal::Closed))
    };

    // ── Type picker ────────────────────────────────────────────────────────────

    let set_webhook = {
        let form_type = form_type.clone();
        let url_err = url_err.clone();
        let chat_id_err = chat_id_err.clone();
        Callback::from(move |_: MouseEvent| {
            form_type.set("webhook".to_string());
            url_err.set(false);
            chat_id_err.set(false);
        })
    };

    let set_slack = {
        let form_type = form_type.clone();
        let url_err = url_err.clone();
        let chat_id_err = chat_id_err.clone();
        Callback::from(move |_: MouseEvent| {
            form_type.set("slack".to_string());
            url_err.set(false);
            chat_id_err.set(false);
        })
    };

    let set_telegram = {
        let form_type = form_type.clone();
        let url_err = url_err.clone();
        let chat_id_err = chat_id_err.clone();
        Callback::from(move |_: MouseEvent| {
            form_type.set("telegram".to_string());
            url_err.set(false);
            chat_id_err.set(false);
        })
    };

    // ── Form input handlers ────────────────────────────────────────────────────

    let on_name = {
        let form_name = form_name.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            form_name.set(el.value());
        })
    };

    let on_webhook_url = {
        let form_webhook_url = form_webhook_url.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            form_webhook_url.set(el.value());
        })
    };

    let on_slack_url = {
        let form_slack_url = form_slack_url.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            form_slack_url.set(el.value());
        })
    };

    let on_telegram_url = {
        let form_telegram_url = form_telegram_url.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            form_telegram_url.set(el.value());
        })
    };

    let on_telegram_chat_id = {
        let form_telegram_chat_id = form_telegram_chat_id.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            form_telegram_chat_id.set(el.value());
        })
    };

    // ── Submit ─────────────────────────────────────────────────────────────────

    let on_submit = {
        let modal = modal.clone();
        let form_name = form_name.clone();
        let form_type = form_type.clone();
        let form_webhook_url = form_webhook_url.clone();
        let form_slack_url = form_slack_url.clone();
        let form_telegram_url = form_telegram_url.clone();
        let form_telegram_chat_id = form_telegram_chat_id.clone();
        let name_err = name_err.clone();
        let url_err = url_err.clone();
        let chat_id_err = chat_id_err.clone();
        let submitting = submitting.clone();
        let modal_alert = modal_alert.clone();
        let reload = reload.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let name_ok = !(*form_name).trim().is_empty();
            let (url_ok, chat_id_ok) = match (*form_type).as_str() {
                "webhook" => (is_valid_url(&form_webhook_url), true),
                "slack" => (is_valid_url(&form_slack_url), true),
                _ => (
                    is_valid_url(&form_telegram_url),
                    !(*form_telegram_chat_id).trim().is_empty(),
                ),
            };

            name_err.set(!name_ok);
            url_err.set(!url_ok);
            chat_id_err.set(!chat_id_ok);

            if !name_ok || !url_ok || !chat_id_ok {
                return;
            }

            let name = (*form_name).trim().to_string();
            let config = match (*form_type).as_str() {
                "webhook" => NotificationChannelConfigInput::Webhook {
                    url: (*form_webhook_url).trim().to_string(),
                },
                "slack" => NotificationChannelConfigInput::Slack {
                    webhook_url: (*form_slack_url).trim().to_string(),
                },
                _ => NotificationChannelConfigInput::Telegram {
                    url: (*form_telegram_url).trim().to_string(),
                    chat_id: (*form_telegram_chat_id).trim().to_string(),
                },
            };
            let data = NotificationChannelFormData { name, config };

            let editing_id = match &*modal {
                ChannelModal::Edit(ch) => Some(ch.id.to_string()),
                _ => None,
            };

            submitting.set(true);
            modal_alert.set(None);

            let modal = modal.clone();
            let submitting = submitting.clone();
            let modal_alert = modal_alert.clone();
            let reload = reload.clone();

            spawn_local(async move {
                let result = if let Some(id) = editing_id {
                    api::update_channel(&id, &data).await
                } else {
                    api::create_channel(&data).await
                };

                match result {
                    Ok(_) => {
                        modal.set(ChannelModal::Closed);
                        reload.emit(());
                    }
                    Err(e) => {
                        modal_alert.set(Some(format!("Failed to save: {e}")));
                        submitting.set(false);
                    }
                }
            });
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

    // ── Header actions ─────────────────────────────────────────────────────────

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

    // ── Derived state for render ───────────────────────────────────────────────

    let modal_open = *modal != ChannelModal::Closed;
    let is_edit = matches!(&*modal, ChannelModal::Edit(_));

    // ── Channel list content ───────────────────────────────────────────────────

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

            // ── Create / Edit modal ────────────────────────────────────────────
            { if modal_open { html! {
                <div class="modal-overlay"
                    onclick={close_modal.clone()}>
                    <div class="modal modal-wide"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                        <h3>{ if is_edit { "Edit channel" } else { "New channel" } }</h3>
                        <div class="modal-subtitle">
                            { if is_edit {
                                "Update this notification channel."
                            } else {
                                "Send alerts to a webhook endpoint, Slack workspace, or Telegram chat."
                            }}
                        </div>

                        { if let Some(msg) = &*modal_alert { html! {
                            <div class="alert alert-error" style="margin-bottom:16px">
                                { msg }
                            </div>
                        }} else { html! {} }}

                        <form onsubmit={on_submit.clone()}>

                            <div class="field" style="margin-bottom:16px">
                                <label>{ "Channel name" }</label>
                                <input
                                    type="text"
                                    placeholder="Ops Slack"
                                    value={(*form_name).clone()}
                                    oninput={on_name}
                                    class={if *name_err { "input-error" } else { "" }}
                                    autocomplete="off"
                                />
                                { if *name_err { html! {
                                    <span class="field-error">{ "Name is required." }</span>
                                }} else { html! {} }}
                            </div>

                            <div class="field" style="margin-bottom:16px">
                                <label>{ "Type" }</label>
                                <div class="type-picker">
                                    <span
                                        class={if (*form_type).as_str() == "webhook" { "type-pill selected" } else { "type-pill" }}
                                        onclick={set_webhook}>
                                        { "Webhook" }
                                    </span>
                                    <span
                                        class={if (*form_type).as_str() == "slack" { "type-pill selected" } else { "type-pill" }}
                                        onclick={set_slack}>
                                        { "Slack" }
                                    </span>
                                    <span
                                        class={if (*form_type).as_str() == "telegram" { "type-pill selected" } else { "type-pill" }}
                                        onclick={set_telegram}>
                                        { "Telegram" }
                                    </span>
                                </div>
                            </div>

                            { match (*form_type).as_str() {
                                "webhook" => html! {
                                    <div class="field" style="margin-bottom:16px">
                                        <label>{ "Webhook URL" }</label>
                                        <input
                                            type="url"
                                            placeholder="https://example.com/webhook"
                                            value={(*form_webhook_url).clone()}
                                            oninput={on_webhook_url}
                                            class={if *url_err { "input-error" } else { "" }}
                                            autocomplete="off"
                                        />
                                        { if *url_err { html! {
                                            <span class="field-error">{ "A valid URL is required." }</span>
                                        }} else { html! {} }}
                                    </div>
                                },
                                "slack" => html! {
                                    <div class="field" style="margin-bottom:16px">
                                        <label>{ "Slack incoming webhook URL" }</label>
                                        <input
                                            type="url"
                                            placeholder="https://hooks.slack.com/services/\u{2026}"
                                            value={(*form_slack_url).clone()}
                                            oninput={on_slack_url}
                                            class={if *url_err { "input-error" } else { "" }}
                                            autocomplete="off"
                                        />
                                        { if *url_err { html! {
                                            <span class="field-error">
                                                { "A valid Slack webhook URL is required." }
                                            </span>
                                        }} else { html! {} }}
                                    </div>
                                },
                                _ => html! {
                                    <>
                                        <div class="field" style="margin-bottom:16px">
                                            <label>{ "Bot API URL" }</label>
                                            <input
                                                type="url"
                                                placeholder="https://api.telegram.org/bot<token>/sendMessage"
                                                value={(*form_telegram_url).clone()}
                                                oninput={on_telegram_url}
                                                class={if *url_err { "input-error" } else { "" }}
                                                autocomplete="off"
                                            />
                                            { if *url_err { html! {
                                                <span class="field-error">
                                                    { "A valid Telegram bot API URL is required." }
                                                </span>
                                            }} else { html! {} }}
                                        </div>
                                        <div class="field" style="margin-bottom:16px">
                                            <label>{ "Chat ID" }</label>
                                            <input
                                                type="text"
                                                placeholder="123456789"
                                                value={(*form_telegram_chat_id).clone()}
                                                oninput={on_telegram_chat_id}
                                                class={if *chat_id_err { "input-error" } else { "" }}
                                                autocomplete="off"
                                            />
                                            { if *chat_id_err { html! {
                                                <span class="field-error">
                                                    { "Chat ID is required." }
                                                </span>
                                            }} else { html! {} }}
                                        </div>
                                    </>
                                },
                            }}

                            <div class="modal-actions">
                                <button type="button" class="btn btn-ghost"
                                    onclick={close_modal.clone()}>
                                    { "Cancel" }
                                </button>
                                <button type="submit" class="btn btn-primary"
                                    disabled={*submitting}>
                                    { if *submitting {
                                        html! { <div class="btn-spinner"></div> }
                                    } else if is_edit {
                                        html! { "Update channel" }
                                    } else {
                                        html! { "Save channel" }
                                    }}
                                </button>
                            </div>

                        </form>
                    </div>
                </div>
            }} else { html! {} }}

            // ── Delete modal ───────────────────────────────────────────────────
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
