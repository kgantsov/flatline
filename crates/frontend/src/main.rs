mod api;
mod components;
mod layout;
mod pages;
mod routes;
mod utils;

use pages::create::CreatePage;
use pages::monitor::MonitorPage;
use pages::monitors::MonitorsPage;
use pages::notifications::NotificationsPage;
use routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

fn switch(route: Route) -> Html {
    match route {
        Route::Monitors => html! { <MonitorsPage /> },
        Route::MonitorDetail { id } => html! { <MonitorPage id={id} /> },
        Route::Create => html! { <CreatePage /> },
        Route::Notifications => html! { <NotificationsPage /> },
        Route::NotFound => html! {
            <div style="text-align:center;padding:64px;color:var(--text-muted)">
                <h1 style="font-size:48px;font-weight:700">{ "404" }</h1>
                <p>{ "Page not found." }</p>
                <a href="/" style="color:var(--accent)">{ "← Back to monitors" }</a>
            </div>
        },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
