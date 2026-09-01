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

    #[at("/status-pages")]
    StatusPages,

    #[at("/status-pages/create")]
    CreateStatusPage,

    #[at("/status-pages/:id/edit")]
    StatusPageEdit { id: String },

    #[at("/status-pages/:id")]
    StatusPageDetail { id: String },

    #[at("/s/:slug")]
    PublicStatus { slug: String },

    #[not_found]
    #[at("/404")]
    NotFound,
}
