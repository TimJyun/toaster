use crate::env::{TOASTER_API_BASE, TOASTER_API_KEY, TOASTER_API_MODEL};
use crate::storage::session::{Session, get_session_store};
use crate::util::sleep::sleep;
use anyhow::{Error, anyhow};
use async_openai_wasm::Client;
use async_openai_wasm::config::OpenAIConfig;
use async_openai_wasm::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestAssistantMessageContentPart, ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPartText, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
};
use chrono::{Duration, Utc};
use dioxus::prelude::{Signal, Task, Writable, spawn};
use futures::StreamExt;
use std::ops::{Add, DerefMut};
use tracing::{debug, error};

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
    session.messages.push(ChatCompletionRequestMessage::User(
        ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(msg),
            name: None,
        },
    ));
    session_store.set(&session_name, &session).await?;

    let request = CreateChatCompletionRequest {
        model: TOASTER_API_MODEL.to_string(),
        messages: session.messages.clone(),
        stream: Some(true),
        ..Default::default()
    };

    let mut stream = client.chat().create_stream(request).await?;

    debug!("spawn stream success");
    Ok(spawn(async move {
        session
            .messages
            .push(ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessage {
                    content: Some(ChatCompletionRequestAssistantMessageContent::Array(
                        Vec::new(),
                    )),
                    ..Default::default()
                },
            ));

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

                    if let Some(ChatCompletionRequestMessage::Assistant(
                        ChatCompletionRequestAssistantMessage {
                            content: Some(ChatCompletionRequestAssistantMessageContent::Array(a)),
                            ..
                        },
                    )) = session.messages.last_mut()
                    {
                        for ccs in ccs_iter {
                            a.push(ccs);
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
        debug!("stream read to end");
    }))
}
