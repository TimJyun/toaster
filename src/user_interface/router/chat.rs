use crate::openai::start_inference;
use crate::user_interface::router::AppRoute;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::user_interface::component::loading::Loading;
use crate::user_interface::component::reload::Reload;
use anyhow::{Error, anyhow};
use chrono::{DateTime, FixedOffset};

use crate::storage::session::get_session_store;

use crate::storage::endpoint::get_endpoint_store;
use crate::tts::tts::TextToSpeech;
use crate::user_interface::app::refresh_app;
use crate::user_interface::fragment::panel::PanelFragment;
use crate::user_interface::fragment::session_list::SessionName;

use async_openai_wasm::types::{
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestAssistantMessageContentPart,
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestUserMessageContentPart,
};
use dioxus::warnings::Warning;
use dioxus_html::KeyCode::T;
use std::ops::Deref;
use std::str::FromStr;
use tracing::{debug, info};

#[component]
pub fn ChatPage(session_id: String) -> Element {
    let session_id = use_memo(use_reactive!(|session_id| { session_id }));

    rsx! {
        PanelFragment { session_id }
    }
}
