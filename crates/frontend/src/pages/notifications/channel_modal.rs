use super::{discord::DiscordForm, slack::SlackForm, telegram::TelegramForm, webhook::WebhookForm};
use crate::api::{
    self, NotificationChannel, NotificationChannelConfig, NotificationChannelConfigInput,
    NotificationChannelFormData,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChannelModalProps {
    /// `None` = create mode, `Some(ch)` = edit mode
    pub channel: Option<NotificationChannel>,
    pub on_close: Callback<()>,
    pub on_saved: Callback<()>,
}

#[function_component(ChannelModal)]
pub fn channel_modal(props: &ChannelModalProps) -> Html {
    let is_edit = props.channel.is_some();

    // Derive initial form values from the channel being edited (or defaults for create).
    // Because this component only mounts while the modal is open, use_state initializers
    // run fresh each time the modal opens — no manual reset needed.
    let (init_name, init_type, init_config) = match &props.channel {
        None => (String::new(), "webhook".to_string(), None),
        Some(ch) => {
            let (type_key, config) = match &ch.config {
                NotificationChannelConfig::Webhook { url } => (
                    "webhook",
                    Some(NotificationChannelConfigInput::Webhook { url: url.clone() }),
                ),
                NotificationChannelConfig::Slack { url } => (
                    "slack",
                    Some(NotificationChannelConfigInput::Slack { url: url.clone() }),
                ),
                NotificationChannelConfig::Telegram { url, chat_id } => (
                    "telegram",
                    Some(NotificationChannelConfigInput::Telegram {
                        url: url.clone(),
                        chat_id: chat_id.clone(),
                    }),
                ),
                NotificationChannelConfig::Discord { url } => (
                    "discord",
                    Some(NotificationChannelConfigInput::Discord { url: url.clone() }),
                ),
            };
            (ch.name.clone(), type_key.to_string(), config)
        }
    };

    let form_name = use_state(|| init_name);
    let form_type = use_state(|| init_type);
    let current_config: UseStateHandle<Option<NotificationChannelConfigInput>> =
        use_state(|| init_config);
    let name_err = use_state(|| false);
    let show_errors = use_state(|| false);
    let submitting = use_state(|| false);
    let alert: UseStateHandle<Option<String>> = use_state(|| None);

    // ── Callbacks ──────────────────────────────────────────────────────────────

    let on_config_change = {
        let current_config = current_config.clone();
        Callback::from(move |config: Option<NotificationChannelConfigInput>| {
            current_config.set(config);
        })
    };

    let on_name = {
        let form_name = form_name.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            form_name.set(el.value());
        })
    };

    let make_set_type = |t: &'static str| {
        let form_type = form_type.clone();
        let current_config = current_config.clone();
        let show_errors = show_errors.clone();
        Callback::from(move |_: MouseEvent| {
            form_type.set(t.to_string());
            current_config.set(None);
            show_errors.set(false);
        })
    };

    let on_submit = {
        let form_name = form_name.clone();
        let current_config = current_config.clone();
        let name_err = name_err.clone();
        let show_errors = show_errors.clone();
        let submitting = submitting.clone();
        let alert = alert.clone();
        let on_saved = props.on_saved.clone();
        let editing_id = props.channel.as_ref().map(|ch| ch.id.to_string());

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let name_ok = !(*form_name).trim().is_empty();
            let config_ok = (*current_config).is_some();

            name_err.set(!name_ok);
            show_errors.set(true);

            if !name_ok || !config_ok {
                return;
            }

            let data = NotificationChannelFormData {
                name: (*form_name).trim().to_string(),
                config: (*current_config).clone().unwrap(),
            };
            let editing_id = editing_id.clone();
            let submitting = submitting.clone();
            let alert = alert.clone();
            let on_saved = on_saved.clone();

            submitting.set(true);
            alert.set(None);

            spawn_local(async move {
                let result = if let Some(id) = editing_id {
                    api::update_channel(&id, &data).await
                } else {
                    api::create_channel(&data).await
                };
                match result {
                    Ok(_) => on_saved.emit(()),
                    Err(e) => {
                        alert.set(Some(format!("Failed to save: {e}")));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    let on_overlay_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };
    let on_cancel = on_overlay_click.clone();

    // ── Sub-form helpers ───────────────────────────────────────────────────────

    let url_from_config = |config: &Option<NotificationChannelConfigInput>| -> String {
        match config {
            Some(
                NotificationChannelConfigInput::Webhook { url }
                | NotificationChannelConfigInput::Slack { url }
                | NotificationChannelConfigInput::Discord { url },
            ) => url.clone(),
            _ => String::new(),
        }
    };

    // ── Render ─────────────────────────────────────────────────────────────────

    html! {
        <div class="modal-overlay" onclick={on_overlay_click}>
            <div class="modal modal-wide"
                onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>

                <h3>{ if is_edit { "Edit channel" } else { "New channel" } }</h3>
                <div class="modal-subtitle">
                    { if is_edit {
                        "Update this notification channel."
                    } else {
                        "Send alerts to a webhook endpoint, Slack workspace, Telegram chat, or Discord server."
                    }}
                </div>

                { if let Some(msg) = &*alert { html! {
                    <div class="alert alert-error" style="margin-bottom:16px">{ msg }</div>
                }} else { html! {} }}

                <form onsubmit={on_submit}>
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
                                onclick={make_set_type("webhook")}>
                                { "Webhook" }
                            </span>
                            <span
                                class={if (*form_type).as_str() == "slack" { "type-pill selected" } else { "type-pill" }}
                                onclick={make_set_type("slack")}>
                                { "Slack" }
                            </span>
                            <span
                                class={if (*form_type).as_str() == "telegram" { "type-pill selected" } else { "type-pill" }}
                                onclick={make_set_type("telegram")}>
                                { "Telegram" }
                            </span>
                            <span
                                class={if (*form_type).as_str() == "discord" { "type-pill selected" } else { "type-pill" }}
                                onclick={make_set_type("discord")}>
                                { "Discord" }
                            </span>
                        </div>
                    </div>

                    { match (*form_type).as_str() {
                        "slack" => html! {
                            <SlackForm
                                initial_url={url_from_config(&current_config)}
                                show_errors={*show_errors}
                                on_change={on_config_change.clone()}
                            />
                        },
                        "telegram" => {
                            let (url, chat_id) = match &*current_config {
                                Some(NotificationChannelConfigInput::Telegram { url, chat_id }) => {
                                    (url.clone(), chat_id.clone())
                                }
                                _ => (String::new(), String::new()),
                            };
                            html! {
                                <TelegramForm
                                    initial_url={url}
                                    initial_chat_id={chat_id}
                                    show_errors={*show_errors}
                                    on_change={on_config_change.clone()}
                                />
                            }
                        },
                        "discord" => html! {
                            <DiscordForm
                                initial_url={url_from_config(&current_config)}
                                show_errors={*show_errors}
                                on_change={on_config_change.clone()}
                            />
                        },
                        _ => html! {
                            <WebhookForm
                                initial_url={url_from_config(&current_config)}
                                show_errors={*show_errors}
                                on_change={on_config_change.clone()}
                            />
                        },
                    }}

                    <div class="modal-actions">
                        <button type="button" class="btn btn-ghost" onclick={on_cancel}>
                            { "Cancel" }
                        </button>
                        <button type="submit" class="btn btn-primary" disabled={*submitting}>
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
    }
}
