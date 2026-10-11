use yew::prelude::*;

/// Config fields specific to TCP monitors: the host and port to connect to.
#[derive(Properties, PartialEq)]
pub(super) struct TcpConfigFieldsProps {
    pub hostname: String,
    pub hostname_err: bool,
    pub port: u16,
    pub port_err: bool,
    pub on_hostname: Callback<InputEvent>,
    pub on_port: Callback<InputEvent>,
}

#[function_component(TcpConfigFields)]
pub(super) fn tcp_config_fields(props: &TcpConfigFieldsProps) -> Html {
    html! {
        <div class="form-card">
            <div class="form-section-title">{ "TCP check" }</div>
            <div class="form-body">

                <div class="field">
                    <label for="f-hostname">{ "Hostname or IP" }</label>
                    <input
                        id="f-hostname"
                        type="text"
                        placeholder="db.example.com"
                        value={props.hostname.clone()}
                        oninput={props.on_hostname.clone()}
                        class={if props.hostname_err { "input-error" } else { "" }}
                        autocomplete="off"
                    />
                    { if props.hostname_err { html! {
                        <span class="field-error">{ "Hostname is required." }</span>
                    }} else { html! {} }}
                </div>

                <div class="field">
                    <label for="f-port">{ "Port" }</label>
                    <input
                        id="f-port"
                        type="number"
                        min="1" max="65535"
                        value={props.port.to_string()}
                        oninput={props.on_port.clone()}
                        class={if props.port_err { "input-error" } else { "" }}
                    />
                    { if props.port_err { html! {
                        <span class="field-error">{ "Must be 1–65535." }</span>
                    }} else { html! {} }}
                </div>

            </div>
        </div>
    }
}
