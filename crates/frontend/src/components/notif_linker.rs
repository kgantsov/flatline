use crate::api::{self, MonitorNotification, NotificationChannel, NotificationChannelConfig};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct NotifLinkerProps {
    pub monitor_id: String,
    pub notifications: Vec<MonitorNotification>,
    pub channels: Vec<NotificationChannel>,
    pub on_reload: Callback<()>,
}

/// Renders the notification channel linking UI (linked channels list + link form).
/// Wrap this in the appropriate card/section container at the call site.
#[function_component(NotifLinker)]
pub fn notif_linker(props: &NotifLinkerProps) -> Html {
    let selected_id = use_state(String::new);
    let on_recovery = use_state(|| true);

    let linked_ids: std::collections::HashSet<_> =
        props.notifications.iter().map(|n| n.channel_id).collect();
    let available: Vec<_> = props.channels.iter().filter(|c| !linked_ids.contains(&c.id)).collect();

    let on_unlink = {
        let mid = props.monitor_id.clone();
        let on_reload = props.on_reload.clone();
        Callback::from(move |channel_id: String| {
            let mid = mid.clone();
            let on_reload = on_reload.clone();
            spawn_local(async move {
                let _ = api::unlink_channel(&mid, &channel_id).await;
                on_reload.emit(());
            });
        })
    };

    let on_link = {
        let mid = props.monitor_id.clone();
        let on_reload = props.on_reload.clone();
        let selected_id = selected_id.clone();
        let on_recovery = on_recovery.clone();
        Callback::from(move |_: MouseEvent| {
            let cid = (*selected_id).clone();
            if cid.is_empty() { return; }
            let mid = mid.clone();
            let on_reload = on_reload.clone();
            let rec = *on_recovery;
            spawn_local(async move {
                let _ = api::link_channel(&mid, &cid, rec).await;
                on_reload.emit(());
            });
        })
    };

    let on_select = {
        let selected_id = selected_id.clone();
        Callback::from(move |e: Event| {
            let el: HtmlSelectElement = e.target_unchecked_into();
            selected_id.set(el.value());
        })
    };

    let on_recovery_change = {
        let on_recovery = on_recovery.clone();
        Callback::from(move |e: Event| {
            let el: HtmlInputElement = e.target_unchecked_into();
            on_recovery.set(el.checked());
        })
    };

    html! {
        <>
            // Linked channels
            <div class="notif-section-label">{ "Linked channels" }</div>
            { if props.notifications.is_empty() {
                html! {
                    <div style="font-size:13px;color:var(--text-muted);padding:4px 0">
                        { "No channels linked yet." }
                    </div>
                }
            } else {
                let rows: Vec<Html> = props.notifications.iter().map(|n| {
                    let ch = props.channels.iter().find(|c| c.id == n.channel_id);
                    let (type_cls, type_label) = match ch.map(|c| &c.config) {
                        Some(NotificationChannelConfig::Slack { .. }) => ("channel-type-badge slack", "Slack"),
                        _ => ("channel-type-badge webhook", "Webhook"),
                    };
                    let ch_name = ch.map(|c| c.name.as_str()).unwrap_or("Unknown");
                    let recovery_color = if n.on_recovery { "var(--up)" } else { "var(--text-muted)" };
                    let recovery_text = if n.on_recovery { "✓ recovery" } else { "– recovery" };
                    let cid = n.channel_id.to_string();
                    let on_unlink = on_unlink.clone();
                    html! {
                        <div class="channel-row">
                            <span class={type_cls}>{ type_label }</span>
                            <span style="flex:1;font-size:13.5px;font-weight:500">{ ch_name }</span>
                            <span style={format!("font-size:12px;color:{}", recovery_color)}>
                                { recovery_text }
                            </span>
                            <button type="button" class="btn btn-danger"
                                style="padding:5px 10px;font-size:12.5px"
                                onclick={Callback::from(move |_: MouseEvent| on_unlink.emit(cid.clone()))}>
                                { "Unlink" }
                            </button>
                        </div>
                    }
                }).collect();
                html! { <div>{ for rows }</div> }
            }}

            // Link new channel
            <div style="margin-top:20px;padding-top:16px;border-top:1px solid var(--border)">
                <div class="notif-section-label">{ "Link a channel" }</div>
                { if props.channels.is_empty() {
                    html! {
                        <div style="font-size:13px;color:var(--text-muted)">
                            { "No notification channels configured yet. " }
                            <a href="/notifications" style="color:var(--accent)">{ "Create one first →" }</a>
                        </div>
                    }
                } else if available.is_empty() {
                    html! {
                        <div style="font-size:13px;color:var(--text-muted)">
                            { "All channels are already linked." }
                        </div>
                    }
                } else {
                    html! {
                        <div class="link-form">
                            <select class="select-input" onchange={on_select}>
                                <option value="">{ "Select a channel…" }</option>
                                { for available.iter().map(|c| {
                                    let label = match &c.config {
                                        NotificationChannelConfig::Slack { .. } => "Slack",
                                        NotificationChannelConfig::Webhook { .. } => "Webhook",
                                    };
                                    html! {
                                        <option value={c.id.to_string()}>
                                            { format!("{} ({})", c.name, label) }
                                        </option>
                                    }
                                })}
                            </select>
                            <label class="checkbox-label">
                                <input type="checkbox"
                                    checked={*on_recovery}
                                    onchange={on_recovery_change}
                                    style="accent-color:var(--accent)" />
                                { "Notify on recovery" }
                            </label>
                            <button type="button" class="btn btn-primary" onclick={on_link}>
                                { "Link" }
                            </button>
                        </div>
                    }
                }}
            </div>
        </>
    }
}
