use yew::prelude::*;

/// Check interval, request timeout, and retry count — common to every
/// monitor type.
#[derive(Properties, PartialEq)]
pub(super) struct TimingFieldsProps {
    pub interval: u32,
    pub interval_err: bool,
    pub timeout_secs: u32,
    pub timeout_err: bool,
    pub retries: u32,
    pub retries_err: bool,
    pub on_interval: Callback<InputEvent>,
    pub on_timeout: Callback<InputEvent>,
    pub on_retries: Callback<InputEvent>,
    pub on_set_interval: Callback<u32>,
    pub on_set_timeout: Callback<u32>,
}

#[function_component(TimingFields)]
pub(super) fn timing_fields(props: &TimingFieldsProps) -> Html {
    let make_set_interval_cb = |v: u32| {
        let cb = props.on_set_interval.clone();
        Callback::from(move |_: MouseEvent| cb.emit(v))
    };

    let make_set_timeout_cb = |v: u32| {
        let cb = props.on_set_timeout.clone();
        Callback::from(move |_: MouseEvent| cb.emit(v))
    };

    html! {
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
    }
}
