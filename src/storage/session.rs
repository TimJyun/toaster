use crate::storage::data_store::DataStore;
use crate::util::string_util::time_to_string;
use async_openai_wasm::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestAssistantMessageContentPart, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
    ChatCompletionRequestSystemMessageContentPart, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart,
};
use chrono::Utc;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub static SESSION_STORE: DataStore<Session> = DataStore::new("sessions");

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Session {
    pub name: String,
    pub speaker: Option<String>,
    pub messages: Vec<Message>,
    pub endpoint: String,
    pub lock_until: Option<chrono::DateTime<chrono::Utc>>,
}

impl Session {
    pub fn is_locking(&self) -> bool {
        if let Some(lock_until) = self.lock_until {
            if lock_until > Utc::now() {
                return true;
            }
        }
        return false;
    }
}

impl Default for Session {
    fn default() -> Self {
        Self {
            name: time_to_string(&chrono::Local::now()),
            speaker: None,
            messages: Vec::new(),
            endpoint: String::new(),
            lock_until: None,
        }
    }
}

pub async fn get_session_store() -> &'static DataStore<Session> {
    &SESSION_STORE
}

pub fn use_session_store() -> &'static DataStore<Session> {
    &SESSION_STORE
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Message {
    pub text: String,
    pub role: Role,

    pub hidden: bool,
    pub filtered: bool,
}

impl TryFrom<ChatCompletionRequestMessage> for Message {
    type Error = &'static str;
    fn try_from(value: ChatCompletionRequestMessage) -> Result<Self, Self::Error> {
        let (text, role) = match value {
            ChatCompletionRequestMessage::System(m) => (
                match m.content {
                    ChatCompletionRequestSystemMessageContent::Text(text) => text,
                    ChatCompletionRequestSystemMessageContent::Array(array) => array
                        .into_iter()
                        .map(|p| match p {
                            ChatCompletionRequestSystemMessageContentPart::Text(text) => text.text,
                        })
                        .collect::<String>(),
                },
                Role::System,
            ),
            ChatCompletionRequestMessage::User(m) => (
                match m.content {
                    ChatCompletionRequestUserMessageContent::Text(text) => text,
                    ChatCompletionRequestUserMessageContent::Array(array) => array
                        .into_iter()
                        .filter_map(|p| match p {
                            ChatCompletionRequestUserMessageContentPart::Text(text) => {
                                Some(text.text)
                            }
                            _ => {
                                debug!("unsupported chat completion message part");
                                None
                            }
                        })
                        .collect::<String>(),
                },
                Role::User,
            ),
            ChatCompletionRequestMessage::Assistant(m) => (
                if let Some(content) = m.content {
                    match content {
                        ChatCompletionRequestAssistantMessageContent::Text(text) => text,
                        ChatCompletionRequestAssistantMessageContent::Array(array) => array
                            .into_iter()
                            .map(|p| match p {
                                ChatCompletionRequestAssistantMessageContentPart::Text(text) => {
                                    text.text
                                }
                                ChatCompletionRequestAssistantMessageContentPart::Refusal(
                                    refusal,
                                ) => refusal.refusal,
                            })
                            .collect::<String>(),
                    }
                } else {
                    return Err("chat completion is empty");
                },
                Role::Assistant,
            ),
            _ => {
                return Err("unsupported chat completion type");
            }
        };

        Ok(Self {
            text,
            role,
            hidden: false,
            filtered: false,
        })
    }
}

impl TryInto<ChatCompletionRequestMessage> for Message {
    type Error = &'static str;

    fn try_into(self) -> Result<ChatCompletionRequestMessage, Self::Error> {
        if !self.filtered {
            Ok(match self.role {
                Role::System => {
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: ChatCompletionRequestSystemMessageContent::Text(self.text),
                        name: None,
                    })
                }
                Role::User => {
                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(self.text),
                        name: None,
                    })
                }
                Role::Assistant => {
                    ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                        content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                            self.text,
                        )),
                        name: None,
                        ..Default::default()
                    })
                }
            })
        } else {
            Err("This message is for viewing by users only")
        }
    }
}
