use crate::storage::session::{Session, get_session_store};
use crate::storage::setting::{Setting, get_setting};
use crate::user_interface::component::confirm_box::ConfirmBox;
use crate::user_interface::component::loading::Loading;
use crate::user_interface::router::AppRoute;
use crate::user_interface::window::{WindowSize, use_window_size_provider};
use async_openai_wasm::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage,
};
use dioxus::core_macro::rsx;
use dioxus::dioxus_core::Element;
use dioxus::document::Link;
use dioxus::hooks::use_resource;
use dioxus::prelude::*;
use dioxus::warnings::Warning;
use dioxus_html::head;
use jwt_compact::UntrustedToken;
use opendal::{Operator, services};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::io;
use std::io::Error;
use tracing::debug;

const _TAILWIND: Asset = asset!("/assets/tailwind.css");
const _CUSTOM: Asset = asset!("/assets/custom.css");

static NEED_UPDATE: GlobalSignal<bool> = Signal::global(|| false);

pub(crate) fn app() -> Element {
    let _init_app = use_future(init);

    let css = rsx! {
        document::Stylesheet { href: _TAILWIND }
        document::Stylesheet { href: _CUSTOM }
    };
    if { *NEED_UPDATE.read() == true } {
        // spawn避免警告
        spawn(async move {
            *NEED_UPDATE.write() = false;
        });
        return rsx! {
            {css}
        };
    }

    let mut window_size_signal = use_window_size_provider();

    rsx! {
        {css}
        div {
            class: "size-full",
            onresize: move |evt| {
                if let Ok(box_size) = evt.get_border_box_size() {
                    window_size_signal
                        .set(WindowSize {
                            width: box_size.width,
                            height: box_size.height,
                        });
                }
            },
            SuspenseBoundary {
                fallback: |context: SuspenseContext| rsx! {
                    Loading {}
                },
                ConfirmBox {}
                Router::<AppRoute> {}
            }
        }
    }
}

async fn init() {
    let setting_store = get_setting();
    let mut setting = setting_store.get().peek().clone();

    let cargo_pkg_version = env!("CARGO_PKG_VERSION");

    if !setting.initialized {
        debug!("initializing");
        let session_store = get_session_store().await;
        let mut session = session_store.get("help").await.unwrap_or_default();
        session
            .messages
            .push(ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessage {
                    content: Some(ChatCompletionRequestAssistantMessageContent::Text(format!(
                        "当前版本：{}",
                        cargo_pkg_version
                    ))),
                    ..Default::default()
                },
            ));
        let _ = session_store.set("help", &session).await;
        debug!("initialize success");
    }

    if setting.current_version != cargo_pkg_version {
        setting.current_version = cargo_pkg_version.to_string();
    }

    let _ = setting_store.set(setting);
}

#[cfg(feature = "web")]
pub(crate) fn reload_page() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}

pub(crate) fn refresh_app() {
    *NEED_UPDATE.write() = true;
}
