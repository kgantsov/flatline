use web_sys::HtmlInputElement;
use yew::prelude::*;

/// Editable list of key/value rows, used for request headers and form-encoded
/// body fields alike.
#[derive(Properties, PartialEq)]
pub(super) struct KvRowsProps {
    pub rows: Vec<(String, String)>,
    pub add_label: String,
    pub on_key: Callback<(usize, String)>,
    pub on_val: Callback<(usize, String)>,
    pub on_remove: Callback<usize>,
    pub on_add: Callback<()>,
}

#[function_component(KvRows)]
pub(super) fn kv_rows(props: &KvRowsProps) -> Html {
    html! {
        <>
            { for props.rows.iter().enumerate().map(|(i, (k, v))| {
                let on_key = {
                    let cb = props.on_key.clone();
                    Callback::from(move |e: InputEvent| {
                        let el: HtmlInputElement = e.target_unchecked_into();
                        cb.emit((i, el.value()));
                    })
                };
                let on_val = {
                    let cb = props.on_val.clone();
                    Callback::from(move |e: InputEvent| {
                        let el: HtmlInputElement = e.target_unchecked_into();
                        cb.emit((i, el.value()));
                    })
                };
                let on_remove = {
                    let cb = props.on_remove.clone();
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
                let cb = props.on_add.clone();
                Callback::from(move |_: MouseEvent| cb.emit(()))
            }}>
                { props.add_label.clone() }
            </button>
        </>
    }
}
