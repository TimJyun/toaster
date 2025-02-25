use crate::openai::start_inference;
use crate::storage::endpoint::get_endpoint_store;
use crate::storage::session::get_session_store;
use crate::user_interface::fragment::markdown::MarkdownFragment;
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
pub fn ChatFragment(session_name: Signal<String>) -> Element {
    let nav = use_navigator();

    let mut session_endpoints_res = use_resource(move || async move {
        let session_storage = get_session_store().await;
        let session_name = session_name.read().to_string();

        let mut session = session_storage.get(session_name).await.unwrap_or_default();

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

    let msg = session_read.messages.iter().filter_map(|m| {
        let mut is_user_sender: bool;
        let msg = match m {
            ChatCompletionRequestMessage::User(m) => {
                is_user_sender = true;
                match &m.content {
                    ChatCompletionRequestUserMessageContent::Text(text) => text.to_string(),
                    ChatCompletionRequestUserMessageContent::Array(a) => {
                        let mut text = String::new();
                        for i in a.iter() {
                            if let ChatCompletionRequestUserMessageContentPart::Text(t) = i {
                                text.push_str(t.text.as_str());
                            }
                        }
                        text
                    }
                }
            }
            ChatCompletionRequestMessage::Assistant(m) => {
                is_user_sender = false;
                match &m.content {
                    Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => {
                        text.to_string()
                    }
                    Some(ChatCompletionRequestAssistantMessageContent::Array(a)) => {
                        let mut text = String::new();
                        for i in a.iter() {
                            if let ChatCompletionRequestAssistantMessageContentPart::Text(t) = i {
                                text.push_str(t.text.as_str());
                            }
                        }
                        text
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };
        let msg_dlg = if is_user_sender {
            let paragraphs = msg.lines().map(|l| {
                rsx! {
                    div { "{l}" }
                }
            });

            rsx! {
                span { class: "flex-1" }
                span { class: "border border-blue-100 bg-blue-100 rounded-xl m-2 p-2",
                    {paragraphs}
                }
            }
        } else {
            let mut think;
            let mut answer;

            if let Some(0) = msg.find("<think>") {
                if let Some(end) = msg.find("</think>") {
                    think = &msg[7..end];
                    answer = &msg[end + 8..msg.len()];
                } else {
                    think = msg.trim_start_matches("<think>");
                    answer = "";
                }
            } else {
                think = "";
                answer = msg.as_str();
            };

            let think_node = if think.trim().is_empty() {
                rsx! {}
            } else {
                let paragraphs = think.lines().map(|l| {
                    rsx! {
                        div { class: "text-xs text-gray-500", {l} }
                    }
                });
                rsx! {
                    details { open: true,
                        summary { class: "text-xs text-gray-500", "思考过程" }
                        {paragraphs}
                    }
                }
            };

            let answer_node = if answer.is_empty() {
                rsx! {}
            } else {
                rsx! {
                    MarkdownFragment { md_text: answer }
                }
            };

            rsx! {
                div { class: "w-full",
                    {think_node}
                    {answer_node}
                }
            }
        };
        rsx! {
            div { class: "flex flex-row", {msg_dlg} }
        }
        .into()
    });

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
                            let session_name = session_name.peek().to_string();
                            if let Ok(mut session) = session_storage.get(&session_name).await {
                                session.endpoint = endpoint;
                                if let Err(err) = session_storage.set(session_name, &session).await {
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
                    initial_value: session_name.to_string(),
                    onchange: move |evt: Event<FormData>| {
                        let new_name = evt.value();
                        async move {
                            if let Err(err) = async move {
                                let session_store = get_session_store().await;
                                let session_name = session_name.peek().to_string();
                                if let Ok(s) = session_store.get(&session_name).await {
                                    session_store.set(&new_name, &s).await?;
                                    let _ = session_store.delete(&session_name).await;
                                    nav.replace(AppRoute::ChatPage {
                                        session_name: new_name,
                                    });
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
            div { class: "flex-1 overflow-y-scroll", {msg} }
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
                        onclick: {
                            let session = session_read.clone();
                            to_owned![session_name];
                            move |_| {
                                to_owned![session_name, session];
                                async move {
                                    to_owned![session_name, session];
                                    let draft = draft_signal.peek().to_string();
                                    let session_name = session_name.peek().to_string();
                                    match start_inference(session_name, draft).await {
                                        Ok(_) => {
                                            *draft_signal.write() = String::new();
                                        }
                                        Err(err) => {
                                            debug!("start failed error : {err}");
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}
