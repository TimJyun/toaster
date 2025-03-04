use crate::env::{TOASTER_API_BASE, TOASTER_API_KEY, TOASTER_API_MODEL};
use crate::storage::session::{Message, Role, Session, get_session_store};
use crate::user_interface::component::toast::make_toast;
use crate::util::sleep::sleep;
use anyhow::{Error, anyhow};
use async_openai_wasm::Client;
use async_openai_wasm::config::OpenAIConfig;
use async_openai_wasm::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestAssistantMessageContentPart, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPartText, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, ChatCompletionResponseStream,
    CreateChatCompletionRequest,
};
use chrono::{Duration, Utc};
use dioxus::prelude::{Signal, Task, Writable, spawn};
use futures::StreamExt;
use std::ops::{Add, DerefMut};
use std::sync::atomic::AtomicIsize;
use tracing::{debug, error};

pub static INFERENCING: AtomicIsize = AtomicIsize::new(0);

pub async fn start_inference(session_name: String, msg: String) -> Result<Task, anyhow::Error> {
    let client = Client::with_config(
        OpenAIConfig::default()
            .with_api_base(TOASTER_API_BASE)
            .with_api_key(TOASTER_API_KEY),
    );

    let session_store = get_session_store().await;
    let mut session = session_store.get(&session_name).await.unwrap_or_default();

    if session.is_locking() {
        return Err(anyhow!("Session is locked"));
    }

    const LOCK_TIMEOUT: i64 = 15;

    session.lock_until = Some(Utc::now() + Duration::seconds(LOCK_TIMEOUT));

    session_store.set(&session_name, &session).await?;

    session.messages.push(Message {
        text: msg,
        role: Role::User,
        hidden: false,
        filtered: false,
    });

    let request = CreateChatCompletionRequest {
        model: TOASTER_API_MODEL.to_string(),
        messages: session
            .messages
            .iter()
            .cloned()
            .filter_map(|m| m.try_into().ok())
            .collect(),
        stream: Some(true),
        ..Default::default()
    };

    // create_stream function is always success
    let mut stream = client.chat().create_stream(request).await?;

    debug!("spawn stream success");
    INFERENCING.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ok(spawn(async move {
        session.messages.push(Message {
            text: String::new(),
            role: Role::Assistant,
            hidden: false,
            filtered: false,
        });

        while let Some(response) = stream.next().await {
            match response {
                Ok(completion_response) => {
                    let ccs_iter = completion_response.choices.into_iter().filter_map(|ccs| {
                        ChatCompletionRequestAssistantMessageContentPart::Text(
                            ChatCompletionRequestMessageContentPartText {
                                text: ccs.delta.content?,
                            },
                        )
                        .into()
                    });
                    if let Some(m) = session.messages.last_mut() {
                        for ccs in ccs_iter {
                            m.text.push_str(
                                match ccs {
                                    ChatCompletionRequestAssistantMessageContentPart::Text(
                                        text,
                                    ) => text.text,
                                    ChatCompletionRequestAssistantMessageContentPart::Refusal(
                                        refusal,
                                    ) => refusal.refusal,
                                }
                                .as_str(),
                            );
                        }
                    } else {
                        unreachable!();
                    }
                    session.lock_until = Some(Utc::now() + Duration::seconds(LOCK_TIMEOUT));
                    let _ = session_store.set(&session_name, &session).await;
                }
                Err(e) => {
                    error!("Error: {:?}", e);
                    break;
                }
            }
        }
        session.lock_until = None;
        let _ = session_store.set(&session_name, &session).await;
        INFERENCING.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        debug!("stream read to end");
    }))
}
