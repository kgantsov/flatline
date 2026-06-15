use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Monitors,

    #[at("/monitors/:id")]
    MonitorDetail { id: String },

    #[at("/create")]
    Create,

    #[at("/notifications")]
    Notifications,

    #[not_found]
    #[at("/404")]
    NotFound,
}
