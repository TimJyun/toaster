use dioxus::prelude::*;
use std::collections::{BTreeMap, HashMap};

use crate::user_interface::router::AppRoute;

use crate::user_interface::component::loading::Loading;

use crate::storage::endpoint::{ModelEndpoint, get_endpoint_store};
use crate::storage::session::{Session, get_session_store};

use crate::tts::tts::TextToSpeech;
use crate::user_interface::component::confirm_box::confirm;
use async_openai_wasm::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent,
};
use futures::{AsyncReadExt, FutureExt};
use std::ops::Deref;
use std::str::FromStr;
use tracing::{debug, info};

#[component]
pub fn EndpointPage() -> Element {
    let list_res = use_resource(|| async {
        let endpoint_store = get_endpoint_store().await;
        let list = endpoint_store.list().await.unwrap_or_default();
        let mut result = BTreeMap::new();
        for n in list.into_iter() {
            if let Ok(e) = endpoint_store.get(&n).await {
                result.insert(n, e);
            }
        }
        result
    })
    .suspend()?;
    let list_read = list_res.read();
    let list_node = list_read.iter().map(move |(n, e)| {
        rsx! {
            div { class: "flex flex-row",
                Link {
                    to: AppRoute::EditeModelPage {
                        endpoint_name: n.to_string(),
                    },
                    "{n}"
                }
                span { class: "flex-1" }
                span {
                    onclick: {
                        to_owned![n];
                        move |_| {
                            to_owned![n];
                            async move {
                                if confirm(vec!["确定要删除该模型？"]).await {
                                    let endpoint = get_endpoint_store().await;
                                    let _ = endpoint.delete(&n).await;
                                }
                            }
                        }
                    },
                    "X"
                }
            }
        }
    });

    rsx! {
        div {
            Link { to: AppRoute::NewModelPage {}, "添加模型" }
        }
        {list_node}
    }
}
