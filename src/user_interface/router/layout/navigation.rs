use dioxus::prelude::*;

use crate::text::TEXT;
use crate::user_interface::router::AppRoute;
use dioxus_router::prelude::{Link, Outlet};

const BOTTOM_NAVIGATION_HEIGHT: usize = 64;
const BOTTOM_NAVIGATION_ITEM_HEIGHT: usize = 32;

pub fn Navigation() -> Element {
    rsx! {
        div { class: "w-full h-full flex flex-col",
            div { class: "flex-1 overflow-auto", Outlet::<AppRoute> {} }
            div { class: "w-full h-16 flex items-center bg-white",
                Link {
                    to: AppRoute::IndexPage {},
                    class: "flex-1 inline-block text-center",
                    "消息"
                }
            }
        }
    }
}
