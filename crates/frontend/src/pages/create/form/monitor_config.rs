use std::collections::HashMap;

use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::api::{HttpBody, HttpBodyInput, MonitorConfig, MonitorConfigInput};

use super::http_config::HttpConfigFields;
use super::tcp_config::TcpConfigFields;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Http,
    Tcp,
}

/// Renders the check-config section for whichever monitor type is selected
/// (HTTP, TCP, ...) and owns all type-specific form state itself — the rest
/// of the create/edit form never needs to know a URL or a port exists.
#[derive(Properties, PartialEq)]
pub(super) struct MonitorConfigFieldsProps {
    /// Existing config to hydrate from in edit mode; `None` when creating.
    pub initial: Option<MonitorConfig>,
    /// Surface field-level errors once the user has tried to submit.
    pub show_errors: bool,
    /// Fires on every change with the current config, or `None` while invalid.
    pub on_change: Callback<Option<MonitorConfigInput>>,
}

#[function_component(MonitorConfigFields)]
pub(super) fn monitor_config_fields(props: &MonitorConfigFieldsProps) -> Html {
    let kind = use_state(|| Kind::Http);

    // HTTP state
    let url = use_state(String::new);
    let method = use_state(|| "GET".to_string());
    let status_codes = use_state(Vec::<u16>::new);
    let tag_text = use_state(String::new);
    let req_headers = use_state(Vec::<(String, String)>::new);
    let body_type = use_state(|| "none".to_string());
    let body_json = use_state(String::new);
    let body_form_fields = use_state(Vec::<(String, String)>::new);

    // TCP state
    let hostname = use_state(String::new);
    let port = use_state(|| 0u16);

    // Hydrate from the fetched monitor in edit mode.
    {
        let kind = kind.clone();
        let url = url.clone();
        let method = method.clone();
        let status_codes = status_codes.clone();
        let req_headers = req_headers.clone();
        let body_type = body_type.clone();
        let body_json = body_json.clone();
        let body_form_fields = body_form_fields.clone();
        let hostname = hostname.clone();
        let port = port.clone();

        use_effect_with(props.initial.clone(), move |initial| {
            if let Some(cfg) = initial {
                match cfg {
                    MonitorConfig::Http { url: u, method: meth, expected_status, headers: hdrs, body: b } => {
                        kind.set(Kind::Http);
                        url.set(u.clone());
                        if let Some(meth) = meth {
                            method.set(meth.to_string());
                        }
                        if let Some(codes) = expected_status
                            && !codes.is_empty() {
                                status_codes.set(codes.clone());
                            }
                        if let Some(hdrs) = hdrs {
                            req_headers.set(hdrs.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
                        }
                        match b {
                            Some(HttpBody::Json { content }) => {
                                body_type.set("json".to_string());
                                body_json.set(serde_json::to_string_pretty(content).unwrap_or_default());
                            }
                            Some(HttpBody::Form { fields }) => {
                                body_type.set("form".to_string());
                                body_form_fields.set(fields.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
                            }
                            None => {}
                        }
                    }
                    MonitorConfig::Tcp { hostname: h, port: p } => {
                        kind.set(Kind::Tcp);
                        hostname.set(h.clone());
                        port.set(*p);
                    }
                }
            }
        });
    }

    // ── Event handlers ────────────────────────────────────────────────────

    let on_kind_select = {
        let kind = kind.clone();
        Callback::from(move |k: Kind| kind.set(k))
    };

    let on_url = { let s = url.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlInputElement = e.target_unchecked_into();
        s.set(el.value());
    })};

    let on_method_select = { let s = method.clone(); Callback::from(move |m: String| s.set(m)) };

    let on_tag_input = { let s = tag_text.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlInputElement = e.target_unchecked_into();
        s.set(el.value());
    })};

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

    let on_body_json = { let s = body_json.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlTextAreaElement = e.target_unchecked_into();
        s.set(el.value());
    })};

    let on_body_type_select = { let s = body_type.clone(); Callback::from(move |t: String| s.set(t)) };

    let on_add_header = {
        let req_headers = req_headers.clone();
        Callback::from(move |_: ()| {
            let mut hdrs = (*req_headers).clone();
            hdrs.push((String::new(), String::new()));
            req_headers.set(hdrs);
        })
    };

    let on_remove_header = {
        let req_headers = req_headers.clone();
        Callback::from(move |i: usize| {
            let hdrs: Vec<_> = (*req_headers).iter().enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, v)| v.clone())
                .collect();
            req_headers.set(hdrs);
        })
    };

    let on_header_key = {
        let req_headers = req_headers.clone();
        Callback::from(move |(i, val): (usize, String)| {
            let mut hdrs = (*req_headers).clone();
            if let Some(row) = hdrs.get_mut(i) { row.0 = val; }
            req_headers.set(hdrs);
        })
    };

    let on_header_val = {
        let req_headers = req_headers.clone();
        Callback::from(move |(i, val): (usize, String)| {
            let mut hdrs = (*req_headers).clone();
            if let Some(row) = hdrs.get_mut(i) { row.1 = val; }
            req_headers.set(hdrs);
        })
    };

    let on_add_form_field = {
        let body_form_fields = body_form_fields.clone();
        Callback::from(move |_: ()| {
            let mut fields = (*body_form_fields).clone();
            fields.push((String::new(), String::new()));
            body_form_fields.set(fields);
        })
    };

    let on_remove_form_field = {
        let body_form_fields = body_form_fields.clone();
        Callback::from(move |i: usize| {
            let fields: Vec<_> = (*body_form_fields).iter().enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, v)| v.clone())
                .collect();
            body_form_fields.set(fields);
        })
    };

    let on_form_field_key = {
        let body_form_fields = body_form_fields.clone();
        Callback::from(move |(i, val): (usize, String)| {
            let mut fields = (*body_form_fields).clone();
            if let Some(row) = fields.get_mut(i) { row.0 = val; }
            body_form_fields.set(fields);
        })
    };

    let on_form_field_val = {
        let body_form_fields = body_form_fields.clone();
        Callback::from(move |(i, val): (usize, String)| {
            let mut fields = (*body_form_fields).clone();
            if let Some(row) = fields.get_mut(i) { row.1 = val; }
            body_form_fields.set(fields);
        })
    };

    let on_hostname = { let s = hostname.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlInputElement = e.target_unchecked_into();
        s.set(el.value());
    })};

    let on_port = { let s = port.clone(); Callback::from(move |e: InputEvent| {
        let el: HtmlInputElement = e.target_unchecked_into();
        if let Ok(v) = el.value().parse::<u16>() { s.set(v); }
    })};

    // ── Derived draft + validity ─────────────────────────────────────────

    let body_json_invalid = *body_type == "json" && {
        let s = (*body_json).trim();
        !s.is_empty() && serde_json::from_str::<serde_json::Value>(s).is_err()
    };

    let draft: Option<MonitorConfigInput> = match *kind {
        Kind::Http => {
            if !super::super::is_valid_url(&url) || body_json_invalid {
                None
            } else {
                let headers_map: HashMap<String, String> = (*req_headers)
                    .iter()
                    .filter(|(k, _)| !k.trim().is_empty())
                    .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                    .collect();

                let body = match (*body_type).as_str() {
                    "json" => {
                        let s = (*body_json).trim();
                        if s.is_empty() {
                            None
                        } else {
                            serde_json::from_str::<serde_json::Value>(s)
                                .ok()
                                .map(|content| HttpBodyInput::Json { content })
                        }
                    }
                    "form" => {
                        let fields: HashMap<String, String> = (*body_form_fields)
                            .iter()
                            .filter(|(k, _)| !k.trim().is_empty())
                            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                            .collect();
                        if fields.is_empty() { None } else { Some(HttpBodyInput::Form { fields }) }
                    }
                    _ => None,
                };

                Some(MonitorConfigInput::Http {
                    url: (*url).trim().to_string(),
                    method: (*method).clone(),
                    expected_status: (*status_codes).clone(),
                    headers: headers_map,
                    body,
                })
            }
        }
        Kind::Tcp => {
            let host = (*hostname).trim();
            if host.is_empty() || *port == 0 {
                None
            } else {
                Some(MonitorConfigInput::Tcp { hostname: host.to_string(), port: *port })
            }
        }
    };

    {
        let on_change = props.on_change.clone();
        use_effect_with(draft.clone(), move |draft| {
            on_change.emit(draft.clone());
        });
    }

    let url_err = props.show_errors && matches!(*kind, Kind::Http) && !super::super::is_valid_url(&url);
    let hostname_err = props.show_errors && matches!(*kind, Kind::Tcp) && (*hostname).trim().is_empty();
    let port_err = props.show_errors && matches!(*kind, Kind::Tcp) && *port == 0;

    let make_kind_cb = |k: Kind| {
        let cb = on_kind_select.clone();
        Callback::from(move |_: MouseEvent| cb.emit(k))
    };

    html! {
        <>
            <div class="form-card">
                <div class="form-section-title">{ "Check type" }</div>
                <div class="form-body">
                    <div class="field">
                        <div class="method-picker">
                            <span class={if *kind == Kind::Http { "method-pill selected" } else { "method-pill" }}
                                onclick={make_kind_cb(Kind::Http)}>{ "HTTP" }</span>
                            <span class={if *kind == Kind::Tcp { "method-pill selected" } else { "method-pill" }}
                                onclick={make_kind_cb(Kind::Tcp)}>{ "TCP" }</span>
                        </div>
                    </div>
                </div>
            </div>

            { match *kind {
                Kind::Http => html! {
                    <HttpConfigFields
                        url={(*url).clone()}
                        url_err={url_err}
                        method={(*method).clone()}
                        status_codes={(*status_codes).clone()}
                        tag_text={(*tag_text).clone()}
                        req_headers={(*req_headers).clone()}
                        body_type={(*body_type).clone()}
                        body_json={(*body_json).clone()}
                        body_json_err={props.show_errors && body_json_invalid}
                        body_form_fields={(*body_form_fields).clone()}
                        on_url={on_url}
                        on_method_select={on_method_select}
                        on_tag_input={on_tag_input}
                        on_tag_keydown={on_tag_keydown}
                        on_remove_code={on_remove_code}
                        on_body_json={on_body_json}
                        on_body_type_select={on_body_type_select}
                        on_add_header={on_add_header}
                        on_remove_header={on_remove_header}
                        on_header_key={on_header_key}
                        on_header_val={on_header_val}
                        on_add_form_field={on_add_form_field}
                        on_remove_form_field={on_remove_form_field}
                        on_form_field_key={on_form_field_key}
                        on_form_field_val={on_form_field_val}
                    />
                },
                Kind::Tcp => html! {
                    <TcpConfigFields
                        hostname={(*hostname).clone()}
                        hostname_err={hostname_err}
                        port={*port}
                        port_err={port_err}
                        on_hostname={on_hostname}
                        on_port={on_port}
                    />
                },
            }}
        </>
    }
}
