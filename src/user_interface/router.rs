use crate::user_interface::router::chat::ChatPage;
use crate::user_interface::router::endpoints::EndpointPage;
use crate::user_interface::router::index::IndexPage;
use crate::user_interface::router::layout::navigation::Navigation;
use crate::user_interface::router::layout::top_navigation::TopNavigation;
use crate::user_interface::router::model::EditeModelPage;
use crate::user_interface::router::model::NewModelPage;
use crate::util::sleep::sleep;
use derive_more::{Display, FromStr};
use dioxus::prelude::*;

use serde::{Deserialize, Serialize};
use std::time::Duration;

mod chat;
mod endpoints;
mod index;
mod layout;
mod model;

#[derive(Routable, PartialEq, Clone, Debug)]
#[rustfmt::skip]
pub enum AppRoute {
    #[route("/")]
    IndexPage{},
    #[route("/chat/:session_name")]
    ChatPage{session_name:String },
    #[layout(Navigation)]
    #[end_layout]
    //
    #[layout(TopNavigation)]
    #[route("/endpoints")]
    EndpointPage{},
    #[route("/model")]
    NewModelPage{},
    #[route("/model/:endpoint_name")]
    EditeModelPage{endpoint_name:String},
    #[end_layout]
    //
    //
    //
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    let nav = use_navigator();
    let _ = use_coroutine(move |_: UnboundedReceiver<()>| async move {
        sleep(2_000).await;
        nav.replace(AppRoute::IndexPage {});
    });
    //todo
    rsx! {
        h1 { "Page Not Found" }
        div { "forward to index page in 2 second" }
    }
}
