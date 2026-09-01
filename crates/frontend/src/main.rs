mod api;
mod components;
mod hooks;
mod layout;
mod pages;
mod routes;
mod utils;

use pages::create::CreatePage;
use pages::create_status_page::CreateStatusPage;
use pages::login::LoginPage;
use pages::monitor::MonitorPage;
use pages::monitors::MonitorsPage;
use pages::notifications::NotificationsPage;
use pages::public_status::PublicStatusPageComponent;
use pages::status_page_detail::StatusPageDetailPage;
use pages::status_pages::StatusPagesPage;
use routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

fn protected_switch(route: Route) -> Html {
    match route {
        Route::Monitors => html! { <MonitorsPage /> },
        Route::MonitorDetail { id } => html! { <MonitorPage id={id} /> },
        Route::Create => html! { <CreatePage /> },
        Route::Notifications => html! { <NotificationsPage /> },
        Route::StatusPages => html! { <StatusPagesPage /> },
        Route::CreateStatusPage => html! { <CreateStatusPage /> },
        Route::StatusPageEdit { id } => html! { <CreateStatusPage edit_id={id} /> },
        Route::StatusPageDetail { id } => html! { <StatusPageDetailPage id={id} /> },
        Route::PublicStatus { slug } => html! { <PublicStatusPageComponent slug={slug} /> },
        Route::NotFound => html! {
            <div style="text-align:center;padding:64px;color:var(--text-muted)">
                <h1 style="font-size:48px;font-weight:700">{ "404" }</h1>
                <p>{ "Page not found." }</p>
                <a href="/" style="color:var(--accent)">{ "← Back to monitors" }</a>
            </div>
        },
    }
}

/// Top-level switch: public status pages are rendered directly, everything
/// else goes through the auth guard.
fn top_switch(route: Route) -> Html {
    match route {
        Route::PublicStatus { slug } => html! { <PublicStatusPageComponent slug={slug} /> },
        other => html! { <AuthGuard route={other} /> },
    }
}

#[derive(Properties, PartialEq)]
struct AuthGuardProps {
    route: Route,
}

/// Checks /auth/me and shows LoginPage when unauthenticated.
/// All hooks are called unconditionally to avoid Yew hook order violations.
#[function_component(AuthGuard)]
fn auth_guard(props: &AuthGuardProps) -> Html {
    let authed = use_state(|| Option::<bool>::None);

    {
        let authed = authed.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::fetch_me().await {
                    Ok(_) => authed.set(Some(true)),
                    Err(_) => authed.set(Some(false)),
                }
            });
        });
    }

    match *authed {
        None => html! {},
        Some(true) => protected_switch(props.route.clone()),
        Some(false) => html! { <LoginPage /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={top_switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
