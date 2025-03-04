use crate::openai::start_inference;
use crate::storage::endpoint::get_endpoint_store;
use crate::storage::session::get_session_store;
use crate::user_interface::component::markdown::Markdown;
use crate::user_interface::fragment::message::MessageFragment;
use crate::user_interface::fragment::session_list::SessionListFragment;
use crate::user_interface::router::AppRoute;
use async_openai_wasm::types::{
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestAssistantMessageContentPart,
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestUserMessageContentPart,
};
use chrono::{Datelike, Timelike};
use dioxus::core_macro::{component, rsx};
use dioxus::dioxus_core::Element;
use dioxus::hooks::{use_reactive, use_resource, use_signal};
use dioxus::prelude::*;
use dioxus_signals::{Readable, Signal, Writable};
use std::ops::Deref;
use tracing::debug;

#[component]
pub fn ChatFragment(session_id: Memo<String>) -> Element {
    let mut session_endpoints_res = use_resource(move || async move {
        let session_storage = get_session_store().await;
        let session_id = session_id.read().to_string();

        let mut session = session_storage.get(session_id).await.unwrap_or_default();

        let endpoint_store = get_endpoint_store().await;
        let endpoints = endpoint_store.list().await;
        let endpoints = endpoints.unwrap_or_default();
        if session.endpoint.is_empty() {
            session.endpoint = endpoints.iter().next().cloned().unwrap_or_default();
        };
        (session, endpoints)
    });

    let mut session_endpoints_signal = session_endpoints_res.suspend()?;

    let mut draft_signal = use_signal(String::new);

    let session_endpoints_read = session_endpoints_signal.read();

    let (session_read, endpoints_read) = session_endpoints_read.deref();

    let session = Readable::map(session_endpoints_signal.clone(), |(s, _)| s);

    let endpoints_node = endpoints_read.iter().map(|e| {
        rsx! {
            option { "{e}" }
        }
    });

    rsx! {
        div { class: "size-full flex flex-col",
            div { class: "flex flex-row",
                span { class: "flex-1" }
                select {
                    class: "hidden",
                    class: "min-w-20",
                    disabled: session_read.is_locking(),
                    value: session_read.endpoint.to_string(),
                    onchange: move |evt: Event<FormData>| {
                        let endpoint = evt.value().to_string();
                        async move {
                            let session_storage = get_session_store().await;
                            let session_id = session_id.peek().to_string();
                            if let Ok(mut session) = session_storage.get(&session_id).await {
                                session.endpoint = endpoint;
                                if let Err(err) = session_storage.set(session_id, &session).await {
                                    debug!("change model failed : {err}");
                                }
                            }
                        }
                    },
                    {endpoints_node}
                }
                input {
                    class: "w-auto min-w-20",
                    disabled: session_read.is_locking(),
                    initial_value: session_read.name.to_string(),
                    onchange: move |evt: Event<FormData>| {
                        let new_name = evt.value();
                        async move {
                            if let Err(err) = async move {
                                let session_store = get_session_store().await;
                                let session_id = session_id.peek().to_string();
                                if let Ok(mut s) = session_store.get(&session_id).await {
                                    s.name = new_name;
                                    session_store.set(&session_id, &s).await?;
                                }
                                anyhow::Ok(())
                            }
                                .await
                            {
                                debug!("session rename error : {err}");
                            }
                        }
                    },
                }
                span { class: "flex-1" }
            }
            div { class: "flex-1 overflow-y-scroll",
                MessageFragment { session }
            }
            div { class: "border border-blue-500 rounded-xl m-2 p-2",
                div {
                    textarea {
                        class: "w-full resize-none min-h-8 focus:outline-none",
                        value: draft_signal,
                        placeholder: if session_read.is_locking() { "AI推理中" } else { "消息" },
                        oninput: move |evt| {
                            draft_signal.set(evt.value());
                        },
                    }
                }
                div { class: "flex flex-row",
                    span { class: "flex-1" }
                    input {
                        class: "disabled:text-gray-400",
                        r#type: "button",
                        disabled: draft_signal.read().is_empty() || session_read.is_locking(),
                        value: "发送",
                        onclick: move |_| async move {
                            let draft = draft_signal.peek().to_string();
                            let session_id = session_id.peek().to_string();
                            match start_inference(session_id.clone(), draft).await {
                                Ok(_) => {
                                    *draft_signal.write() = String::new();
                                }
                                Err(err) => {
                                    debug!("start failed error : {err}");
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}
