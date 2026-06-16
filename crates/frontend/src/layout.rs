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
                <div class="header-right">
                    { if let Some(actions) = props.header_actions.clone() {
                        html! { <div class="header-actions">{ actions }</div> }
                    } else {
                        html! {}
                    }}
                    <form method="post" action="/auth/logout" style="margin:0">
                        <button type="submit" class="btn-icon" title="Logout">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                                stroke-linecap="round" stroke-linejoin="round"
                                style="width:18px;height:18px">
                                <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
                                <polyline points="16 17 21 12 16 7"/>
                                <line x1="21" y1="12" x2="9" y2="12"/>
                            </svg>
                        </button>
                    </form>
                </div>
            </header>

            { for props.children.iter() }

            <footer>{ "flatline \u{2014} open-source uptime monitoring" }</footer>
        </div>
    }
}
