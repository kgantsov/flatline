use yew::prelude::*;

#[derive(PartialEq, Clone, Copy, Default)]
pub enum NavActive {
    #[default]
    None,
    Monitors,
    Notifications,
}

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    #[prop_or_default]
    pub active: NavActive,
    #[prop_or_default]
    pub header_actions: Option<Html>,
    pub children: Children,
}

#[function_component(Layout)]
pub fn layout(props: &LayoutProps) -> Html {
    let monitors_cls = if props.active == NavActive::Monitors { "active" } else { "" };
    let notifications_cls = if props.active == NavActive::Notifications { "active" } else { "" };

    html! {
        <div class="app">
            <header>
                <a href="/" class="logo">
                    <span class="logo-icon">
                        <svg viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="2.5"
                            stroke-linecap="round" stroke-linejoin="round"
                            style="width:16px;height:16px">
                            <polyline points="3 12 6 12 9 4 12 20 15 12 18 12 21 12"/>
                        </svg>
                    </span>
                    { "flatline" }
                </a>
                <nav>
                    <a href="/" class={monitors_cls}>{ "Monitors" }</a>
                    <a href="/notifications" class={notifications_cls}>{ "Notifications" }</a>
                </nav>
                { if let Some(actions) = props.header_actions.clone() {
                    html! { <div class="header-actions">{ actions }</div> }
                } else {
                    html! {}
                }}
            </header>

            { for props.children.iter() }

            <footer>{ "flatline \u{2014} open-source uptime monitoring" }</footer>
        </div>
    }
}
