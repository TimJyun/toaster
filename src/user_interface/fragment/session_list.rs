use crate::imgs::DELETE;
use dioxus::prelude::*;
use std::collections::HashMap;

use crate::user_interface::router::AppRoute;

use crate::user_interface::component::loading::Loading;

use crate::storage::session::{get_session_store, use_session_store};

use crate::user_interface::component::confirm_box::confirm;
use async_openai_wasm::types::{
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestAssistantMessageContentPart,
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestUserMessageContentPart,
};
use futures::{AsyncReadExt, FutureExt};
use std::ops::Deref;
use std::str::FromStr;
use tracing::{debug, info};

pub type SessionName = String;

#[component]
pub fn SessionListFragment() -> Element {
    let session_store = use_session_store();

    let mut name_with_session_res = use_resource(|| async {
        let sessions = session_store.list().await.unwrap_or_default();
        let mut result = Vec::with_capacity(sessions.len());
        for n in sessions.into_iter() {
            if let Ok(session) = session_store.get(&n).await {
                result.push((n, session));
            };
        }
        result
    });
    let name_with_session = name_with_session_res.suspend()?;
    let name_with_session_read = name_with_session.read();
    let x = name_with_session_read.iter().map(|(id, s)| {
        let m = s
            .messages
            .last()
            .map(|m| match m {
                ChatCompletionRequestMessage::User(m) => {
                    let txt = match &m.content {
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
                    };
                    format!("您：{txt}")
                }
                ChatCompletionRequestMessage::Assistant(m) => match &m.content {
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
                    _ => String::new(),
                },
                _ => "<无聊天记录>".to_string(),
            })
            .unwrap_or("<无聊天记录>".to_string());

        let m = m.chars().take(120).collect::<String>();
        let n = &s.name;

        rsx! {
            div { class: "odd:bg-gray-200",
                div { class: "flex flex-row",
                    Link {
                        to: AppRoute::ChatPage {
                            session_id: id.to_string(),
                        },
                        "{n}"
                    }
                    span { class: "flex-1" }
                    input {
                        class: "size-8 px-2",
                        class: if s.is_locking() { "hidden" },
                        disabled: s.is_locking(),
                        r#type: "image",
                        alt: "delete",
                        src: DELETE,
                        onclick: {
                            to_owned![id];
                            move |_| {
                                to_owned![id];
                                async move {
                                    if confirm(vec!["您确定要删掉该会话？"]).await {
                                        let session_storage = get_session_store().await;
                                        if let Ok(_) = session_storage.delete(id).await {
                                            name_with_session_res.restart();
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
                Link {
                    class: "text-xs text-gray-500",
                    to: AppRoute::ChatPage {
                        session_id: id.to_string(),
                    },
                    {m}
                }
            }
        }
    });

    rsx! {
        div { class: "h-full flex flex-col",
            div { class: "flex flex-row m-1",
                Link { to: AppRoute::IndexPage {},
                    span { class: "border rounded-xl p-1", "新会话" }
                }
                span { class: "flex-1" }
                Link { to: AppRoute::EndpointPage {}, "管理模型" }
            }
            div { class: "flex-1 overflow-y-scroll space-y-2", {x} }
        }
    }
}
