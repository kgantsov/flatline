mod http_config;
mod kv_rows;
mod monitor_config;
mod tcp_config;
mod timing;

use crate::api::{MonitorConfig, MonitorConfigInput, MonitorNotification, NotificationChannel};
use crate::components::NotifLinker;
use yew::prelude::*;

use monitor_config::MonitorConfigFields;
use timing::TimingFields;

// ── CreateForm ────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub(super) struct CreateFormProps {
    // form values (generic, apply to every monitor type)
    pub name: String,
    pub interval: u32,
    pub timeout_secs: u32,
    pub retries: u32,
    pub enabled: bool,
    // validation
    pub name_err: bool,
    pub interval_err: bool,
    pub timeout_err: bool,
    pub retries_err: bool,
    // meta
    pub is_edit: bool,
    pub title: String,
    pub desc: String,
    pub submit_label: String,
    pub back_href: String,
    pub submitting: bool,
    pub alert: Option<super::Alert>,
    // callbacks
    pub on_name: Callback<InputEvent>,
    pub on_interval: Callback<InputEvent>,
    pub on_timeout: Callback<InputEvent>,
    pub on_retries: Callback<InputEvent>,
    pub on_enabled: Callback<Event>,
    pub on_set_interval: Callback<u32>,
    pub on_set_timeout: Callback<u32>,
    pub on_submit: Callback<SubmitEvent>,
    // check config — type-specific (HTTP, TCP, ...) fields and validation
    // live entirely inside MonitorConfigFields; this page only forwards the
    // hydration source and gets the finished value back.
    pub initial_config: Option<MonitorConfig>,
    pub show_config_errors: bool,
    pub on_config_change: Callback<Option<MonitorConfigInput>>,
    // notifications (edit mode)
    pub edit_id: Option<String>,
    pub notifications: Vec<MonitorNotification>,
    pub all_channels: Vec<NotificationChannel>,
    pub on_reload_notifs: Callback<()>,
}

#[function_component(CreateForm)]
pub(super) fn create_form(props: &CreateFormProps) -> Html {
    html! {
        <main class="narrow">
            <div class="breadcrumb">
                <a href="/">{ "Monitors" }</a>
                <span class="breadcrumb-sep">{ "/" }</span>
                <span>{ &props.title }</span>
            </div>

            <div class="page-title">{ &props.title }</div>
            <div class="page-desc">{ &props.desc }</div>

            { match &props.alert {
                Some(super::Alert::Error(msg)) => html! {
                    <div class="alert alert-error">{ msg }</div>
                },
                None => html! {},
            }}

            <form onsubmit={props.on_submit.clone()} novalidate=true>

                // ── General ───────────────────────────────────────────────
                <div class="form-card">
                    <div class="form-section-title">{ "General" }</div>
                    <div class="form-body">

                        <div class="field">
                            <label for="f-name">{ "Monitor name" }</label>
                            <input
                                id="f-name"
                                type="text"
                                placeholder="Production API"
                                value={props.name.clone()}
                                oninput={props.on_name.clone()}
                                class={if props.name_err { "input-error" } else { "" }}
                                autocomplete="off"
                            />
                            { if props.name_err { html! {
                                <span class="field-error">{ "Name is required." }</span>
                            }} else { html! {} }}
                        </div>

                        <div class="toggle-field">
                            <div class="toggle-info">
                                <div class="toggle-label-text">{ "Enable monitor" }</div>
                                <div class="toggle-description">
                                    { "Start monitoring immediately after saving." }
                                </div>
                            </div>
                            <label class="switch">
                                <input type="checkbox" checked={props.enabled} onchange={props.on_enabled.clone()} />
                                <span class="switch-track"></span>
                            </label>
                        </div>

                    </div>
                </div>

                // ── Check config (per monitor type) ─────────────────────────
                <MonitorConfigFields
                    initial={props.initial_config.clone()}
                    show_errors={props.show_config_errors}
                    on_change={props.on_config_change.clone()}
                />

                // ── Timing ────────────────────────────────────────────────
                <TimingFields
                    interval={props.interval}
                    interval_err={props.interval_err}
                    timeout_secs={props.timeout_secs}
                    timeout_err={props.timeout_err}
                    retries={props.retries}
                    retries_err={props.retries_err}
                    on_interval={props.on_interval.clone()}
                    on_timeout={props.on_timeout.clone()}
                    on_retries={props.on_retries.clone()}
                    on_set_interval={props.on_set_interval.clone()}
                    on_set_timeout={props.on_set_timeout.clone()}
                />

                // ── Actions ───────────────────────────────────────────────
                <div class="form-actions">
                    <a href={props.back_href.clone()} class="btn btn-ghost">{ "Cancel" }</a>
                    <button type="submit" class="btn btn-primary" disabled={props.submitting}>
                        { if props.submitting {
                            html! { <div class="btn-spinner"></div> }
                        } else {
                            html! {
                                <>
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                        stroke-width="2.5" stroke-linecap="round"
                                        stroke-linejoin="round" style="width:15px;height:15px">
                                        <polyline points="20 6 9 17 4 12"/>
                                    </svg>
                                    { props.submit_label.clone() }
                                </>
                            }
                        }}
                    </button>
                </div>

            </form>

            // ── Notifications (edit mode only) ────────────────────────────
            { if props.is_edit {
                let mid = props.edit_id.clone().unwrap_or_default();
                html! {
                    <div class="form-card" style="margin-top:16px">
                        <div class="form-section-title">{ "Notification channels" }</div>
                        <div class="form-body">
                            <NotifLinker
                                monitor_id={mid}
                                notifications={props.notifications.clone()}
                                channels={props.all_channels.clone()}
                                on_reload={props.on_reload_notifs.clone()}
                            />
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}

        </main>
    }
}
