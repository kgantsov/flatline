use super::kv_rows::KvRows;
use yew::prelude::*;

const METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];

fn url_preview(url: &str) -> Html {
    let url = url.trim();
    if url.is_empty() {
        return html! { <div class="url-preview"></div> };
    }
    if super::super::is_valid_url(url) {
        let without_proto = url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        html! { <div class="url-preview valid">{ without_proto }</div> }
    } else {
        html! { <div class="url-preview invalid">{ "Invalid URL" }</div> }
    }
}

/// Config fields specific to HTTP monitors: URL, method, expected status
/// codes, request headers, and request body.
#[derive(Properties, PartialEq)]
pub(super) struct HttpConfigFieldsProps {
    pub url: String,
    pub url_err: bool,
    pub method: String,
    pub status_codes: Vec<u16>,
    pub tag_text: String,
    pub req_headers: Vec<(String, String)>,
    pub body_type: String,
    pub body_json: String,
    pub body_json_err: bool,
    pub body_form_fields: Vec<(String, String)>,
    pub on_url: Callback<InputEvent>,
    pub on_method_select: Callback<String>,
    pub on_tag_input: Callback<InputEvent>,
    pub on_tag_keydown: Callback<KeyboardEvent>,
    pub on_remove_code: Callback<u16>,
    pub on_body_json: Callback<InputEvent>,
    pub on_body_type_select: Callback<String>,
    pub on_add_header: Callback<()>,
    pub on_remove_header: Callback<usize>,
    pub on_header_key: Callback<(usize, String)>,
    pub on_header_val: Callback<(usize, String)>,
    pub on_add_form_field: Callback<()>,
    pub on_remove_form_field: Callback<usize>,
    pub on_form_field_key: Callback<(usize, String)>,
    pub on_form_field_val: Callback<(usize, String)>,
}

#[function_component(HttpConfigFields)]
pub(super) fn http_config_fields(props: &HttpConfigFieldsProps) -> Html {
    let make_method_cb = |m: &'static str| {
        let cb = props.on_method_select.clone();
        Callback::from(move |_: MouseEvent| cb.emit(m.to_string()))
    };

    let make_remove_code_cb = |code: u16| {
        let cb = props.on_remove_code.clone();
        Callback::from(move |_: MouseEvent| cb.emit(code))
    };

    html! {
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
                    <KvRows
                        rows={props.req_headers.clone()}
                        add_label={"+ Add header".to_string()}
                        on_key={props.on_header_key.clone()}
                        on_val={props.on_header_val.clone()}
                        on_remove={props.on_remove_header.clone()}
                        on_add={props.on_add_header.clone()}
                    />
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
                            <KvRows
                                rows={props.body_form_fields.clone()}
                                add_label={"+ Add field".to_string()}
                                on_key={props.on_form_field_key.clone()}
                                on_val={props.on_form_field_val.clone()}
                                on_remove={props.on_remove_form_field.clone()}
                                on_add={props.on_add_form_field.clone()}
                            />
                        }
                    } else {
                        html! {}
                    }}
                </div>

            </div>
        </div>
    }
}
