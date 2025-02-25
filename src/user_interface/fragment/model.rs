use dioxus::prelude::*;
use std::collections::HashMap;

use crate::user_interface::router::AppRoute;

use crate::user_interface::component::loading::Loading;

use crate::storage::endpoint::{ModelEndpoint, get_endpoint_store};
use crate::storage::session::{Session, get_session_store};

use crate::tts::tts::TextToSpeech;
use crate::user_interface::client_util::go_back_or_replace_to_index;
use crate::user_interface::component::confirm_box::confirm;
use async_openai_wasm::Client;
use async_openai_wasm::config::OpenAIConfig;
use async_openai_wasm::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent,
};
use futures::{AsyncReadExt, FutureExt};
use std::ops::Deref;
use std::str::FromStr;
use tracing::{debug, info};

#[component]
pub fn ModelFragment(endpoint_name: String) -> Element {
    let nav = use_navigator();

    let mut busying = use_signal(|| false);

    let create_mode = endpoint_name.is_empty();
    let mut name = use_signal(use_reactive!(|endpoint_name| endpoint_name));
    let mut endpoint_res = use_resource(use_reactive!(|endpoint_name| async move {
        let endpoint_store = get_endpoint_store().await;
        Signal::new(endpoint_store.get(&endpoint_name).await.unwrap_or_default())
    }));

    let endpoint = *endpoint_res.suspend()?.read().deref();

    rsx! {
        div {
            label { class: "flex flex-row",
                span { class: "w-16", "名字：" }
                input {
                    class: "border w-48",
                    placeholder: "不能为空或重复",
                    disabled: !create_mode,
                    onchange: move |evt| {
                        name.set(evt.value());
                    },
                }
            }
        }
        div {
            OpenaiConfigurator { config: endpoint }
        }
        div {
            input {
                class: "border disabled:text-gray-400",
                r#type: "button",
                disabled: *busying.read() || name.read().is_empty(),
                value: "保存",
                onclick: move |_| {
                    busying.set(true);
                    async move {
                        let name = name.peek().to_string();
                        let config = endpoint.peek().clone();
                        let endpoint_store = get_endpoint_store().await;
                        if (!create_mode) || endpoint_store.get(&name).await.is_err()
                            || confirm(vec!["模型已存在，是否覆盖"]).await
                        {
                            match endpoint_store.set(name, &config).await {
                                Ok(()) => {
                                    go_back_or_replace_to_index(nav);
                                    debug!("save success ");
                                }
                                Err(err) => {
                                    debug!("save failed : {err}");
                                }
                            }
                        }
                        busying.set(false);
                    }
                },
            }
        }
    }
}

#[component]
pub fn OpenaiConfigurator(mut config: Signal<ModelEndpoint>) -> Element {
    let models = use_resource(move || async move {
        let config_read = config.read();
        if false == (config_read.base.is_empty() || config_read.key.is_empty()) {
            let client = Client::with_config(
                OpenAIConfig::default()
                    .with_api_base(&config_read.base)
                    .with_api_key(&config_read.key),
            );
            drop(config_read);

            let models = client.models().list().await;

            return models.ok();
        }
        None
    });

    let model_node = {
        let models_read = models.read();
        let model_options = models_read
            .as_ref()
            .map(|o| {
                o.as_ref().map(|m| {
                    m.data.iter().map(|m| {
                        rsx! {
                            option { value: "{m.id}" }
                        }
                    })
                })
            })
            .flatten();
        let model_options_node = if let Some(model_options) = model_options {
            rsx! {
                {model_options}
            }
        } else {
            rsx! {}
        };

        rsx! {
            input {
                class: "border w-48",
                initial_value: config.read().model.to_string(),
                list: "model-options",
                onchange: move |evt| {
                    config.write().model = evt.value();
                },
            }
            datalist { id: "model-options", {model_options_node} }
        }
    };

    rsx! {
        div {
            label {
                span { class: "inline-block w-16", "base : " }
                input {
                    class: "border w-64",
                    initial_value: config.read().base.to_string(),
                    onchange: move |evt| {
                        config.write().base = evt.value();
                    },
                }
            }
        }
        div {
            label {
                span { class: "inline-block w-16", "key : " }
                input {
                    class: "border w-64",
                    initial_value: config.read().key.to_string(),
                    onchange: move |evt| {
                        config.write().key = evt.value();
                    },
                }
            }
        }
        div {
            label {
                span { class: "inline-block w-16", "model : " }
                {model_node}
            }
        }
    }
}
