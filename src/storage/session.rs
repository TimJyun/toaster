use crate::storage::data_store::DataStore;
use crate::util::string_util::time_to_string;
use async_openai_wasm::types::ChatCompletionRequestMessage;
use chrono::Utc;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub static SESSION_STORE: DataStore<Session> = DataStore::new("sessions");

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Session {
    pub name: String,
    pub speaker: Option<String>,
    pub messages: Vec<ChatCompletionRequestMessage>,
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
