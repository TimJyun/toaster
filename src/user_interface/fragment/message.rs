use crate::storage::session::{Role, Session};
use crate::user_interface::component::markdown::Markdown;
use async_openai_wasm::types::{
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestAssistantMessageContentPart,
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestUserMessageContentPart,
};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn MessageFragment(session: MappedSignal<Session>) -> Element {
    let session_read = session.read();

    let msg = session_read.messages.iter().filter_map(|m| {
        let mut is_user_sender: bool = m.role == Role::User;
        let msg = &m.text;
        let msg_dlg = if is_user_sender {
            rsx! {
                span { class: "flex-1" }
                span { class: "border border-blue-100 bg-blue-100 rounded-xl m-2 p-2",
                    MessageBox { text: msg }
                }
            }
        } else {
            rsx! {
                MessageBox { text: msg }
            }
        };
        rsx! {
            div { class: "flex flex-row", {msg_dlg} }
        }
        .into()
    });

    rsx! {
        {msg}
    }
}
#[derive(Clone, PartialEq)]
pub struct MessageRef {
    pub session_id: Uuid,
    pub message_id: Uuid,
}

#[component]
pub fn MessageBox(message_ref: Option<MessageRef>, text: String) -> Element {
    let Text { think, main } = parse_text(text.as_str());

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

    let main_node = if main.is_empty() {
        rsx! {}
    } else {
        rsx! {
            Markdown { md_text: main }
        }
    };

    rsx! {
        div { class: "w-full",
            {think_node}
            {main_node}
        }
    }
}

struct Text<'a> {
    think: &'a str,
    main: &'a str,
}

fn parse_text(text: &str) -> Text {
    let think: &str;
    let main: &str;
    if let Some(0) = text.find("<think>") {
        if let Some(end) = text.find("</think>") {
            think = &text[7..end];
            main = &text[end + 8..text.len()];
        } else {
            think = text.trim_start_matches("<think>");
            main = "";
        }
    } else {
        think = "";
        main = text;
    };
    Text { think, main }
}
