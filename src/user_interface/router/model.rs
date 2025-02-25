use crate::user_interface::fragment::model::ModelFragment;
use dioxus::core_macro::component;
use dioxus::dioxus_core::Element;
use dioxus::prelude::*;

#[component]
pub fn EditeModelPage(endpoint_name: String) -> Element {
    rsx! {
        ModelFragment { endpoint_name }
    }
}

#[component]
pub fn NewModelPage() -> Element {
    rsx! {
        ModelFragment { endpoint_name: String::new() }
    }
}
