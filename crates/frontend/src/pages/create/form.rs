use crate::api::{MonitorNotification, NotificationChannel};
use crate::components::NotifLinker;
use web_sys::HtmlInputElement;
use yew::prelude::*;

const METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];

fn url_preview(url: &str) -> Html {
    let url = url.trim();
    if url.is_empty() {
        return html! { <div class="url-preview"></div> };
    }
    if super::is_valid_url(url) {
        let without_proto = url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        html! { <div class="url-preview valid">{ without_proto }</div> }
    } else {
        html! { <div class="url-preview invalid">{ "Invalid URL" }</div> }
    }
}

// ── CreateForm ────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub(super) struct CreateFormProps {
    // form values
    pub name: String,
    pub url: String,
    pub method: String,
    pub status_codes: Vec<u16>,
    pub tag_text: String,
    pub interval: u32,
    pub timeout_secs: u32,
    pub retries: u32,
    pub enabled: bool,
    // validation
    pub name_err: bool,
    pub url_err: bool,
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
    pub on_url: Callback<InputEvent>,
    pub on_method_select: Callback<String>,
    pub on_interval: Callback<InputEvent>,
    pub on_timeout: Callback<InputEvent>,
    pub on_retries: Callback<InputEvent>,
    pub on_enabled: Callback<Event>,
    pub on_tag_input: Callback<InputEvent>,
    pub on_tag_keydown: Callback<KeyboardEvent>,
    pub on_remove_code: Callback<u16>,
    pub on_set_interval: Callback<u32>,
    pub on_set_timeout: Callback<u32>,
    pub on_submit: Callback<SubmitEvent>,
    // notifications (edit mode)
    pub edit_id: Option<String>,
    pub notifications: Vec<MonitorNotification>,
    pub all_channels: Vec<NotificationChannel>,
    pub on_reload_notifs: Callback<()>,
    // request headers
    pub req_headers: Vec<(String, String)>,
    pub on_add_header: Callback<()>,
    pub on_remove_header: Callback<usize>,
    pub on_header_key: Callback<(usize, String)>,
    pub on_header_val: Callback<(usize, String)>,
    // request body
    pub body_type: String,
    pub body_json: String,
    pub body_json_err: bool,
    pub body_form_fields: Vec<(String, String)>,
    pub on_body_json: Callback<InputEvent>,
    pub on_body_type_select: Callback<String>,
    pub on_add_form_field: Callback<()>,
    pub on_remove_form_field: Callback<usize>,
    pub on_form_field_key: Callback<(usize, String)>,
    pub on_form_field_val: Callback<(usize, String)>,
}

#[function_component(CreateForm)]
pub(super) fn create_form(props: &CreateFormProps) -> Html {
    let make_method_cb = |m: &'static str| {
        let cb = props.on_method_select.clone();
        Callback::from(move |_: MouseEvent| cb.emit(m.to_string()))
    };

    let make_remove_code_cb = |code: u16| {
        let cb = props.on_remove_code.clone();
        Callback::from(move |_: MouseEvent| cb.emit(code))
    };

    let make_set_interval_cb = |v: u32| {
        let cb = props.on_set_interval.clone();
        Callback::from(move |_: MouseEvent| cb.emit(v))
    };

    let make_set_timeout_cb = |v: u32| {
        let cb = props.on_set_timeout.clone();
        Callback::from(move |_: MouseEvent| cb.emit(v))
    };

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

                // ── HTTP check ────────────────────────────────────────────
                <div class="form-card">
                    <div class="form-section-title">{ "HTTP check" }</div>
                    <div class="form-body">

                        <div class="field">
                            <label for="f-url">
                                { "URL " }
                                <span class="label-hint">{ "must be absolute" }</span>
                            </label>
                            <input
                                id="f-url"
                                type="url"
                                placeholder="https://api.example.com/health"
                                value={props.url.clone()}
                                oninput={props.on_url.clone()}
                                class={if props.url_err { "input-error" } else { "" }}
                                autocomplete="off"
                            />
                            { url_preview(&props.url) }
                            { if props.url_err { html! {
                                <span class="field-error">
                                    { "A valid URL is required (must start with http:// or https://)." }
                                </span>
                            }} else { html! {} }}
                        </div>

                        <div class="field">
                            <label>{ "HTTP method" }</label>
                            <div class="method-picker">
                                { for METHODS.iter().map(|&m| {
                                    let cls = if props.method == m { "method-pill selected" } else { "method-pill" };
                                    html! {
                                        <span class={cls} onclick={make_method_cb(m)}>{ m }</span>
                                    }
                                })}
                            </div>
                        </div>

                        <div class="field">
                            <label>
                                { "Expected status codes " }
                                <span class="label-hint">{ "press Enter or comma to add" }</span>
                            </label>
                            <div class="tags-input">
                                { for props.status_codes.iter().map(|&code| html! {
                                    <span class="tag">
                                        { code.to_string() }
                                        <span class="tag-remove" onclick={make_remove_code_cb(code)}>
                                            { "×" }
                                        </span>
                                    </span>
                                })}
                                <input
                                    type="text"
                                    class="tags-text-input"
                                    placeholder={if props.status_codes.is_empty() { "200" } else { "" }}
                                    value={props.tag_text.clone()}
                                    oninput={props.on_tag_input.clone()}
                                    onkeydown={props.on_tag_keydown.clone()}
                                    inputmode="numeric"
                                    maxlength="3"
                                />
                            </div>
                            <div class="field-hint">
                                { "Leave empty to accept any 2xx. Common codes: 200, 201, 204." }
                            </div>
                        </div>

                        <div class="field">
                            <label>{ "Request headers" }</label>
                            { for props.req_headers.iter().enumerate().map(|(i, (k, v))| {
                                let on_key = {
                                    let cb = props.on_header_key.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let el: HtmlInputElement = e.target_unchecked_into();
                                        cb.emit((i, el.value()));
                                    })
                                };
                                let on_val = {
                                    let cb = props.on_header_val.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let el: HtmlInputElement = e.target_unchecked_into();
                                        cb.emit((i, el.value()));
                                    })
                                };
                                let on_remove = {
                                    let cb = props.on_remove_header.clone();
                                    Callback::from(move |_: MouseEvent| cb.emit(i))
                                };
                                html! {
                                    <div key={i.to_string()} class="kv-row">
                                        <input type="text" placeholder="Name" value={k.clone()}
                                            oninput={on_key} autocomplete="off" />
                                        <input type="text" placeholder="Value" value={v.clone()}
                                            oninput={on_val} autocomplete="off" />
                                        <button type="button" class="kv-remove" onclick={on_remove}>
                                            { "×" }
                                        </button>
                                    </div>
                                }
                            })}
                            <button type="button" class="btn btn-ghost btn-sm" onclick={{
                                let cb = props.on_add_header.clone();
                                Callback::from(move |_: MouseEvent| cb.emit(()))
                            }}>
                                { "+ Add header" }
                            </button>
                        </div>

                        <div class="field">
                            <label>{ "Request body" }</label>
                            <div class="method-picker">
                                { for [("none", "None"), ("json", "JSON"), ("form", "Form")].iter().map(|&(t, label)| {
                                    let cb = props.on_body_type_select.clone();
                                    let cls = if props.body_type == t { "method-pill selected" } else { "method-pill" };
                                    html! {
                                        <span class={cls} onclick={Callback::from(move |_: MouseEvent| cb.emit(t.to_string()))}>
                                            { label }
                                        </span>
                                    }
                                })}
                            </div>
                            { if props.body_type == "json" {
                                html! {
                                    <>
                                    <textarea
                                        class={if props.body_json_err { "input-error" } else { "" }}
                                        placeholder="{\"key\": \"value\"}"
                                        value={props.body_json.clone()}
                                        oninput={props.on_body_json.clone()}
                                        rows="5"
                                        style="font-family: monospace; resize: vertical; margin-top: 8px;"
                                    />
                                    { if props.body_json_err { html! {
                                        <span class="field-error">{ "Invalid JSON." }</span>
                                    }} else { html! {} }}
                                    </>
                                }
                            } else if props.body_type == "form" {
                                html! {
                                    <>
                                    { for props.body_form_fields.iter().enumerate().map(|(i, (k, v))| {
                                        let on_key = {
                                            let cb = props.on_form_field_key.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let el: HtmlInputElement = e.target_unchecked_into();
                                                cb.emit((i, el.value()));
                                            })
                                        };
                                        let on_val = {
                                            let cb = props.on_form_field_val.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let el: HtmlInputElement = e.target_unchecked_into();
                                                cb.emit((i, el.value()));
                                            })
                                        };
                                        let on_remove = {
                                            let cb = props.on_remove_form_field.clone();
                                            Callback::from(move |_: MouseEvent| cb.emit(i))
                                        };
                                        html! {
                                            <div key={i.to_string()} class="kv-row">
                                                <input type="text" placeholder="Name" value={k.clone()}
                                                    oninput={on_key} autocomplete="off" />
                                                <input type="text" placeholder="Value" value={v.clone()}
                                                    oninput={on_val} autocomplete="off" />
                                                <button type="button" class="kv-remove" onclick={on_remove}>
                                                    { "×" }
                                                </button>
                                            </div>
                                        }
                                    })}
                                    <button type="button" class="btn btn-ghost btn-sm" onclick={{
                                        let cb = props.on_add_form_field.clone();
                                        Callback::from(move |_: MouseEvent| cb.emit(()))
                                    }}>
                                        { "+ Add field" }
                                    </button>
                                    </>
                                }
                            } else {
                                html! {}
                            }}
                        </div>

                    </div>
                </div>

                // ── Timing ────────────────────────────────────────────────
                <div class="form-card">
                    <div class="form-section-title">{ "Timing" }</div>
                    <div class="form-body">

                        <div class="field-row field-row-2">
                            <div class="field">
                                <label for="f-interval">
                                    { "Check interval " }
                                    <span class="label-hint">{ "seconds" }</span>
                                </label>
                                <input
                                    id="f-interval"
                                    type="number"
                                    min="10" max="86400"
                                    value={props.interval.to_string()}
                                    oninput={props.on_interval.clone()}
                                    class={if props.interval_err { "input-error" } else { "" }}
                                />
                                <div class="preset-row">
                                    <button type="button" class="preset-btn" onclick={make_set_interval_cb(30)}>{ "30s" }</button>
                                    <button type="button" class="preset-btn" onclick={make_set_interval_cb(60)}>{ "1m" }</button>
                                    <button type="button" class="preset-btn" onclick={make_set_interval_cb(300)}>{ "5m" }</button>
                                    <button type="button" class="preset-btn" onclick={make_set_interval_cb(600)}>{ "10m" }</button>
                                    <button type="button" class="preset-btn" onclick={make_set_interval_cb(1800)}>{ "30m" }</button>
                                </div>
                                { if props.interval_err { html! {
                                    <span class="field-error">{ "Must be 10–86400 seconds." }</span>
                                }} else { html! {} }}
                            </div>

                            <div class="field">
                                <label for="f-timeout">
                                    { "Request timeout " }
                                    <span class="label-hint">{ "seconds" }</span>
                                </label>
                                <input
                                    id="f-timeout"
                                    type="number"
                                    min="1" max="300"
                                    value={props.timeout_secs.to_string()}
                                    oninput={props.on_timeout.clone()}
                                    class={if props.timeout_err { "input-error" } else { "" }}
                                />
                                <div class="preset-row">
                                    <button type="button" class="preset-btn" onclick={make_set_timeout_cb(5)}>{ "5s" }</button>
                                    <button type="button" class="preset-btn" onclick={make_set_timeout_cb(10)}>{ "10s" }</button>
                                    <button type="button" class="preset-btn" onclick={make_set_timeout_cb(30)}>{ "30s" }</button>
                                </div>
                                { if props.timeout_err { html! {
                                    <span class="field-error">{ "Must be 1–300 seconds." }</span>
                                }} else { html! {} }}
                            </div>
                        </div>

                        <div class="field">
                            <label for="f-retries">
                                { "Retries " }
                                <span class="label-hint">{ "before marking down" }</span>
                            </label>
                            <input
                                id="f-retries"
                                type="number"
                                min="0" max="10"
                                value={props.retries.to_string()}
                                oninput={props.on_retries.clone()}
                                class={if props.retries_err { "input-error" } else { "" }}
                            />
                            <div class="field-hint">
                                { "How many consecutive failures before opening an incident. 0 means the first failure triggers it." }
                            </div>
                            { if props.retries_err { html! {
                                <span class="field-error">{ "Must be 0–10." }</span>
                            }} else { html! {} }}
                        </div>

                    </div>
                </div>

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
