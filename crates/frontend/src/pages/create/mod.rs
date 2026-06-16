mod form;

use crate::api::{self, MonitorConfigInput, MonitorFormData, MonitorNotification, NotificationChannel};
use crate::layout::Layout;
use crate::routes::Route;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, InputEvent, KeyboardEvent};
use yew::prelude::*;
use yew_router::prelude::*;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_edit_id(search: &str) -> Option<String> {
    search
        .trim_start_matches('?')
        .split('&')
        .find_map(|part| {
            let mut kv = part.splitn(2, '=');
            let key = kv.next()?;
            let val = kv.next().unwrap_or("").to_string();
            (key == "id" && !val.is_empty()).then_some(val)
        })
}

pub(super) fn is_valid_url(s: &str) -> bool {
    let s = s.trim();
    (s.starts_with("http://") || s.starts_with("https://")) && s.len() > 8
}

// ── Alert ─────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub(super) enum Alert {
    Error(String),
}

// ── Create/Edit page ──────────────────────────────────────────────────────────

#[function_component(CreatePage)]
pub fn create_page() -> Html {
    let location = use_location().unwrap();
    let edit_id: Option<String> = parse_edit_id(location.query_str());
    let is_edit = edit_id.is_some();
    let navigator = use_navigator().unwrap();

    // ── Form state ────────────────────────────────────────────────────────────
    let name = use_state(String::new);
    let url = use_state(String::new);
    let method = use_state(|| "GET".to_string());
    let status_codes = use_state(Vec::<u16>::new);
    let tag_text = use_state(String::new);
    let interval = use_state(|| 60u32);
    let timeout_secs = use_state(|| 10u32);
    let retries = use_state(|| 0u32);
    let enabled = use_state(|| true);

    let name_err = use_state(|| false);
    let url_err = use_state(|| false);
    let interval_err = use_state(|| false);
    let timeout_err = use_state(|| false);
    let retries_err = use_state(|| false);

    let submitting = use_state(|| false);
    let alert = use_state(|| None::<Alert>);

    let notifications = use_state(Vec::<MonitorNotification>::new);
    let all_channels = use_state(Vec::<NotificationChannel>::new);

    // Load existing monitor in edit mode
    {
        let edit_id = edit_id.clone();
        let name = name.clone();
        let url = url.clone();
        let method = method.clone();
        let status_codes = status_codes.clone();
        let interval = interval.clone();
        let timeout_secs = timeout_secs.clone();
        let retries = retries.clone();
        let enabled = enabled.clone();
        let alert = alert.clone();
        let notifications = notifications.clone();
        let all_channels = all_channels.clone();

        use_effect_with(edit_id.clone(), move |edit_id| {
            let Some(id) = edit_id.clone() else { return; };
            spawn_local(async move {
                match api::fetch_monitor(&id).await {
                    Ok(m) => {
                        name.set(m.name.clone());
                        match &m.config {
                            crate::api::MonitorConfig::Http { url: u, method: meth, expected_status } => {
                                url.set(u.clone());
                                if let Some(meth) = meth {
                                    method.set(meth.to_string());
                                }
                                if let Some(codes) = expected_status
                                    && !codes.is_empty() {
                                        status_codes.set(codes.clone());
                                    }
                            }
                        }
                        interval.set(m.interval);
                        timeout_secs.set(m.timeout);
                        retries.set(m.retries);
                        enabled.set(m.enabled);

                        let (notifs, channels) = (
                            api::fetch_monitor_notifications(&id).await,
                            api::fetch_all_channels().await,
                        );
                        notifications.set(notifs);
                        all_channels.set(channels);
                    }
                    Err(e) => {
                        alert.set(Some(Alert::Error(format!("Failed to load monitor: {e}"))));
                    }
                }
            });
        });
    }

    let on_reload_notifs = {
        let edit_id = edit_id.clone();
        let notifications = notifications.clone();
        let all_channels = all_channels.clone();
        Callback::from(move |_: ()| {
            let Some(id) = edit_id.clone() else { return; };
            let notifications = notifications.clone();
            let all_channels = all_channels.clone();
            spawn_local(async move {
                notifications.set(api::fetch_monitor_notifications(&id).await);
                all_channels.set(api::fetch_all_channels().await);
            });
        })
    };

    // ── Event handlers ────────────────────────────────────────────────────────

    let on_name = { let s = name.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlInputElement = e.target_unchecked_into();
        s.set(el.value());
    })};

    let on_url = { let s = url.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlInputElement = e.target_unchecked_into();
        s.set(el.value());
    })};

    let on_interval = { let s = interval.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlInputElement = e.target_unchecked_into();
        if let Ok(v) = el.value().parse::<u32>() { s.set(v); }
    })};

    let on_timeout = { let s = timeout_secs.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlInputElement = e.target_unchecked_into();
        if let Ok(v) = el.value().parse::<u32>() { s.set(v); }
    })};

    let on_retries = { let s = retries.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlInputElement = e.target_unchecked_into();
        if let Ok(v) = el.value().parse::<u32>() { s.set(v); }
    })};

    let on_enabled = { let s = enabled.clone(); Callback::from(move |e: Event| {
        let el: HtmlInputElement = e.target_unchecked_into();
        s.set(el.checked());
    })};

    let on_method_select = {
        let method = method.clone();
        Callback::from(move |m: String| method.set(m))
    };

    let on_tag_input = {
        let tag_text = tag_text.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            tag_text.set(el.value());
        })
    };

    let on_tag_keydown = {
        let status_codes = status_codes.clone();
        let tag_text = tag_text.clone();
        Callback::from(move |e: KeyboardEvent| {
            match e.key().as_str() {
                "Enter" | "," => {
                    e.prevent_default();
                    let val = (*tag_text).trim().to_string();
                    if let Ok(n) = val.parse::<u16>()
                        && (100..=599).contains(&n) {
                            let mut codes = (*status_codes).clone();
                            if !codes.contains(&n) { codes.push(n); }
                            status_codes.set(codes);
                        }
                    tag_text.set(String::new());
                }
                "Backspace" if tag_text.is_empty() => {
                    let mut codes = (*status_codes).clone();
                    codes.pop();
                    status_codes.set(codes);
                }
                _ => {}
            }
        })
    };

    let on_remove_code = {
        let status_codes = status_codes.clone();
        Callback::from(move |code: u16| {
            let codes: Vec<u16> = (*status_codes).iter().filter(|&&c| c != code).cloned().collect();
            status_codes.set(codes);
        })
    };

    let on_set_interval = {
        let interval = interval.clone();
        Callback::from(move |v: u32| interval.set(v))
    };

    let on_set_timeout = {
        let timeout_secs = timeout_secs.clone();
        Callback::from(move |v: u32| timeout_secs.set(v))
    };

    // ── Submit ────────────────────────────────────────────────────────────────
    let on_submit = {
        let name = name.clone();
        let url = url.clone();
        let method = method.clone();
        let status_codes = status_codes.clone();
        let interval = interval.clone();
        let timeout_secs = timeout_secs.clone();
        let retries = retries.clone();
        let enabled = enabled.clone();
        let name_err = name_err.clone();
        let url_err = url_err.clone();
        let interval_err = interval_err.clone();
        let timeout_err = timeout_err.clone();
        let retries_err = retries_err.clone();
        let submitting = submitting.clone();
        let alert = alert.clone();
        let edit_id = edit_id.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let n_ok = !(*name).trim().is_empty();
            let u_ok = is_valid_url(&url);
            let iv = *interval;
            let iv_ok = (10..=86400).contains(&iv);
            let to = *timeout_secs;
            let to_ok = (1..=300).contains(&to);
            let rt = *retries;
            let rt_ok = rt <= 10;

            name_err.set(!n_ok);
            url_err.set(!u_ok);
            interval_err.set(!iv_ok);
            timeout_err.set(!to_ok);
            retries_err.set(!rt_ok);

            if !n_ok || !u_ok || !iv_ok || !to_ok || !rt_ok { return; }

            submitting.set(true);
            alert.set(None);

            let data = MonitorFormData {
                name: (*name).trim().to_string(),
                interval: *interval,
                timeout: *timeout_secs,
                retries: *retries,
                enabled: *enabled,
                config: MonitorConfigInput::Http {
                    url: (*url).trim().to_string(),
                    method: (*method).clone(),
                    expected_status: (*status_codes).clone(),
                },
            };

            let submitting = submitting.clone();
            let alert = alert.clone();
            let edit_id = edit_id.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                let result = if let Some(id) = edit_id {
                    api::update_monitor_full(&id, &data).await
                } else {
                    api::create_monitor(&data).await
                };

                match result {
                    Ok(m) => {
                        navigator.push(&Route::MonitorDetail { id: m.id.to_string() });
                    }
                    Err(e) => {
                        alert.set(Some(Alert::Error(format!("Failed to save monitor: {e}"))));
                        submitting.set(false);
                    }
                }
            });
        })
    };

    // ── Render ────────────────────────────────────────────────────────────────

    let title = if is_edit { "Edit monitor" } else { "Add monitor" };
    let desc = if is_edit {
        "Update the configuration for this monitor."
    } else {
        "Configure a new endpoint to watch. Flatline will ping it on your chosen interval."
    };
    let submit_label = if is_edit { "Update monitor" } else { "Save monitor" };
    let back_href = edit_id.as_deref()
        .map(|id| format!("/monitors/{id}"))
        .unwrap_or_else(|| "/".to_string());

    html! {
        <Layout>
            <form::CreateForm
                name={(*name).clone()}
                url={(*url).clone()}
                method={(*method).clone()}
                status_codes={(*status_codes).clone()}
                tag_text={(*tag_text).clone()}
                interval={*interval}
                timeout_secs={*timeout_secs}
                retries={*retries}
                enabled={*enabled}
                name_err={*name_err}
                url_err={*url_err}
                interval_err={*interval_err}
                timeout_err={*timeout_err}
                retries_err={*retries_err}
                is_edit={is_edit}
                title={title.to_string()}
                desc={desc.to_string()}
                submit_label={submit_label.to_string()}
                back_href={back_href}
                submitting={*submitting}
                alert={(*alert).clone()}
                on_name={on_name}
                on_url={on_url}
                on_method_select={on_method_select}
                on_interval={on_interval}
                on_timeout={on_timeout}
                on_retries={on_retries}
                on_enabled={on_enabled}
                on_tag_input={on_tag_input}
                on_tag_keydown={on_tag_keydown}
                on_remove_code={on_remove_code}
                on_set_interval={on_set_interval}
                on_set_timeout={on_set_timeout}
                on_submit={on_submit}
                edit_id={edit_id.clone()}
                notifications={(*notifications).clone()}
                all_channels={(*all_channels).clone()}
                on_reload_notifs={on_reload_notifs}
            />
        </Layout>
    }
}
