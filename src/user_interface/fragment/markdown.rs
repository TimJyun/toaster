use crate::util::sleep::sleep;
use dioxus::core_macro::component;
use dioxus::dioxus_core::Element;
use dioxus::prelude::*;
use dioxus::warnings::Warning;
use dioxus_signals::Signal;
use std::borrow::Cow;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[component]
pub fn MarkdownFragment(md_text: String) -> Element {
    let html = markdown::to_html(md_text.as_str());
    rsx! {
        div{
            class:"markdown-block",
            dangerous_inner_html:html,
        }
    }
}
