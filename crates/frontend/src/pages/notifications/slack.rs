use crate::api::NotificationChannelConfigInput;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SlackFormProps {
    pub initial_url: String,
    pub show_errors: bool,
    pub on_change: Callback<Option<NotificationChannelConfigInput>>,
}

#[function_component(SlackForm)]
pub fn slack_form(props: &SlackFormProps) -> Html {
    let url = use_state(|| props.initial_url.clone());

    // Emit initial config on mount
    {
        let on_change = props.on_change.clone();
        let url = url.clone();
        use_effect_with((), move |_| on_change.emit(build_config(&url)));
    }

    let on_url = {
        let url = url.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let el: HtmlInputElement = e.target_unchecked_into();
            let val = el.value();
            on_change.emit(build_config(&val));
            url.set(val);
        })
    };

    let url_err = props.show_errors && !super::is_valid_url(&url);

    html! {
        <div class="field" style="margin-bottom:16px">
            <label>{ "Slack incoming webhook URL" }</label>
            <input
                type="url"
                placeholder="https://hooks.slack.com/services/\u{2026}"
                value={(*url).clone()}
                oninput={on_url}
                class={if url_err { "input-error" } else { "" }}
                autocomplete="off"
            />
            { if url_err { html! {
                <span class="field-error">{ "A valid Slack webhook URL is required." }</span>
            }} else { html! {} }}
        </div>
    }
}

fn build_config(url: &str) -> Option<NotificationChannelConfigInput> {
    super::is_valid_url(url).then(|| NotificationChannelConfigInput::Slack {
        webhook_url: url.trim().to_string(),
    })
}
