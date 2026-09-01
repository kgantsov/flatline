use crate::api::{self, StatusPageFormData};
use crate::layout::{Layout, NavActive};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    /// When set, the page acts as an edit form for this status page ID.
    #[prop_or_default]
    pub edit_id: Option<String>,
}

#[function_component(CreateStatusPage)]
pub fn create_status_page(props: &Props) -> Html {
    let name = use_state(String::new);
    let description = use_state(String::new);
    let slug = use_state(String::new);
    let refresh_interval = use_state(|| 60u32);
    let alert = use_state(|| Option::<String>::None);
    let submitting = use_state(|| false);

    let is_edit = props.edit_id.is_some();
    let edit_id = props.edit_id.clone();

    {
        let name = name.clone();
        let description = description.clone();
        let slug = slug.clone();
        let refresh_interval = refresh_interval.clone();
        let edit_id = edit_id.clone();
        use_effect_with(edit_id.clone(), move |eid| {
            if let Some(id) = eid.clone() {
                spawn_local(async move {
                    if let Ok(page) = api::fetch_status_page(&id).await {
                        name.set(page.name);
                        description.set(page.description.unwrap_or_default());
                        slug.set(page.slug);
                        refresh_interval.set(page.refresh_interval);
                    }
                });
            }
        });
    }

    let on_name = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            name.set(input.value());
        })
    };

    let on_desc = {
        let description = description.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            description.set(input.value());
        })
    };

    let on_slug = {
        let slug = slug.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            // Sanitize: lowercase, replace spaces with dashes, strip invalid chars
            let raw = input.value();
            let sanitized: String = raw
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
                .collect();
            slug.set(sanitized);
        })
    };

    let on_interval = {
        let refresh_interval = refresh_interval.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(v) = input.value().parse::<u32>() {
                refresh_interval.set(v);
            }
        })
    };

    let on_submit = {
        let name = name.clone();
        let description = description.clone();
        let slug = slug.clone();
        let refresh_interval = refresh_interval.clone();
        let alert = alert.clone();
        let submitting = submitting.clone();
        let edit_id = edit_id.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let name_val = (*name).clone();
            let desc_val = (*description).clone();
            let slug_val = (*slug).clone();
            let interval = *refresh_interval;
            let alert = alert.clone();
            let submitting = submitting.clone();
            let edit_id = edit_id.clone();

            if name_val.is_empty() || slug_val.is_empty() {
                alert.set(Some("Name and slug are required.".into()));
                return;
            }

            submitting.set(true);
            alert.set(None);

            spawn_local(async move {
                let data = StatusPageFormData {
                    name: name_val,
                    description: desc_val,
                    slug: slug_val,
                    refresh_interval: interval,
                };

                let result = if let Some(id) = &edit_id {
                    api::update_status_page(id, &data).await.map(|_| id.clone())
                } else {
                    api::create_status_page(&data).await.map(|p| p.id.to_string())
                };

                submitting.set(false);
                match result {
                    Ok(id) => {
                        let href = format!("/status-pages/{id}");
                        web_sys::window().unwrap().location().set_href(&href).unwrap();
                    }
                    Err(e) => alert.set(Some(e)),
                }
            });
        })
    };

    let (title, desc_text) = if is_edit {
        ("Edit status page", "Update the settings for this status page.")
    } else {
        ("New status page", "Create a public status page to share monitor status with your users.")
    };

    let slug_hint = if (*slug).is_empty() {
        "The public URL will be /s/<slug>".to_string()
    } else {
        format!("Public URL: /s/{}", *slug)
    };

    html! {
        <Layout active={NavActive::StatusPages}>
            <main class="narrow">
                <div class="breadcrumb">
                    <a href="/status-pages">{ "Status Pages" }</a>
                    <span class="breadcrumb-sep">{ "/" }</span>
                    <span>{ title }</span>
                </div>

                <div class="page-title">{ title }</div>
                <div class="page-desc">{ desc_text }</div>

                { if let Some(msg) = &*alert { html! {
                    <div class="alert alert-error">{ msg }</div>
                }} else { html! {} }}

                <form onsubmit={on_submit} novalidate=true>
                    <div class="form-card">
                        <div class="form-section-title">{ "Page settings" }</div>
                        <div class="form-body">

                            <div class="field">
                                <label for="sp-name">{ "Name" }</label>
                                <input
                                    id="sp-name"
                                    type="text"
                                    placeholder="My Status Page"
                                    value={(*name).clone()}
                                    oninput={on_name}
                                    autocomplete="off"
                                />
                            </div>

                            <div class="field">
                                <label for="sp-desc">
                                    { "Description " }
                                    <span class="label-hint">{ "optional" }</span>
                                </label>
                                <input
                                    id="sp-desc"
                                    type="text"
                                    placeholder="All systems operational"
                                    value={(*description).clone()}
                                    oninput={on_desc}
                                    autocomplete="off"
                                />
                            </div>

                            <div class="field">
                                <label for="sp-slug">{ "Slug" }</label>
                                <input
                                    id="sp-slug"
                                    type="text"
                                    placeholder="my-status-page"
                                    value={(*slug).clone()}
                                    oninput={on_slug}
                                    autocomplete="off"
                                />
                                <span class="label-hint" style="margin-top:4px;display:block">
                                    { slug_hint }
                                </span>
                            </div>

                            <div class="field">
                                <label for="sp-interval">
                                    { "Refresh interval " }
                                    <span class="label-hint">{ "seconds" }</span>
                                </label>
                                <input
                                    id="sp-interval"
                                    type="number"
                                    min="10"
                                    value={refresh_interval.to_string()}
                                    oninput={on_interval}
                                />
                            </div>

                        </div>
                    </div>

                    <div class="form-actions">
                        <a href="/status-pages" class="btn btn-ghost">{ "Cancel" }</a>
                        <button type="submit" class="btn btn-primary" disabled={*submitting}>
                            { if *submitting { "Saving…" } else if is_edit { "Save changes" } else { "Create page" } }
                        </button>
                    </div>
                </form>
            </main>
        </Layout>
    }
}
