use dioxus::prelude::*;
use std::collections::HashMap;

use crate::user_interface::router::AppRoute;

use crate::user_interface::component::loading::Loading;

use crate::storage::session::get_session_store;

use crate::storage::endpoint::get_endpoint_store;
use crate::user_interface::fragment::session_list::SessionListFragment;

use crate::user_interface::fragment::panel::PanelFragment;
use async_openai_wasm::types::{
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestAssistantMessageContentPart,
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageContent,
    ChatCompletionRequestUserMessageContentPart,
};
use chrono::{Datelike, Timelike};
use futures::{AsyncReadExt, FutureExt};
use std::ops::Deref;
use std::str::FromStr;
use tracing::{debug, info};

pub fn now() -> String {
    let time = chrono::Local::now();
    format!(
        "{:02}{:02}{:02}-{:02}:{:02}",
        time.year() % 2000,
        time.month(),
        time.day(),
        time.hour(),
        time.minute(),
    )
}

pub fn IndexPage() -> Element {
    let mut session_name = use_signal(now);

    rsx! {
        PanelFragment { session_name }
    }
}
