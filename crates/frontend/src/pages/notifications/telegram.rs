use crate::api::NotificationChannelConfigInput;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TelegramFormProps {
    pub initial_url: String,
    pub initial_chat_id: String,
    pub show_errors: bool,
    pub on_change: Callback<Option<NotificationChannelConfigInput>>,
}

#[function_component(TelegramForm)]
pub fn telegram_form(props: &TelegramFormProps) -> Html {
    let url = use_state(|| props.initial_url.clone());
    let chat_id = use_state(|| props.initial_chat_id.clone());

    // Emit initial config on mount
    {
        let on_change = props.on_change.clone();
        let url = url.clone();
        let chat_id = chat_id.clone();
        use_effect_with((), move |_| on_change.emit(build_config(&url, &chat_id)));
    }

    let on_url = {
        let url = url.clone();
        let chat_id = chat_id.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            let val = el.value();
            on_change.emit(build_config(&val, &chat_id));
            url.set(val);
        })
    };

    let on_chat_id = {
        let url = url.clone();
        let chat_id = chat_id.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            let val = el.value();
            on_change.emit(build_config(&url, &val));
            chat_id.set(val);
        })
    };

    let url_err = props.show_errors && !super::is_valid_url(&url);
    let chat_id_err = props.show_errors && (*chat_id).trim().is_empty();

    html! {
        <>
            <div class="field" style="margin-bottom:16px">
                <label>{ "Bot API URL" }</label>
                <input
                    type="url"
                    placeholder="https://api.telegram.org/bot<token>/sendMessage"
                    value={(*url).clone()}
                    oninput={on_url}
                    class={if url_err { "input-error" } else { "" }}
                    autocomplete="off"
                />
                { if url_err { html! {
                    <span class="field-error">{ "A valid Telegram bot API URL is required." }</span>
                }} else { html! {} }}
            </div>
            <div class="field" style="margin-bottom:16px">
                <label>{ "Chat ID" }</label>
                <input
                    type="text"
                    placeholder="123456789"
                    value={(*chat_id).clone()}
                    oninput={on_chat_id}
                    class={if chat_id_err { "input-error" } else { "" }}
                    autocomplete="off"
                />
                { if chat_id_err { html! {
                    <span class="field-error">{ "Chat ID is required." }</span>
                }} else { html! {} }}
            </div>
        </>
    }
}

fn build_config(url: &str, chat_id: &str) -> Option<NotificationChannelConfigInput> {
    if super::is_valid_url(url) && !chat_id.trim().is_empty() {
        Some(NotificationChannelConfigInput::Telegram {
            url: url.trim().to_string(),
            chat_id: chat_id.trim().to_string(),
        })
    } else {
        None
    }
}
