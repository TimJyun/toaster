use std::time::Duration;

use dioxus::prelude::*;
use tracing::debug;

use crate::util::sleep::sleep;

static TOAST_BOX_TEXT: GlobalSignal<Vec<ToastOption>> = GlobalSignal::new(|| Vec::new());

pub struct ToastOption {
    text: String,
    //millisecond
    show_time: u32,
}

pub(crate) fn ToastBox() -> Element {
    let mut text = use_signal(String::new);
    let _toast_player = use_future(move || async move {
        loop {
            sleep(50).await;
            let toast = TOAST_BOX_TEXT.write().pop();
            if let Some(toast) = toast {
                text.set(toast.text);
                sleep(toast.show_time).await;
                text.set(String::new());
            }
        }
    });

    if text.read().is_empty() {
        rsx! {}
    } else {
        rsx! {
            div { style: "
                    position: fixed;
                    text-align: center;
                    width: 100%;
                    bottom: 20vh;
                ",
                span { style: "
                    background-color: rgba(0,0,0,0.5);
                    padding: 4px;
                    border-radius: 0.5rem;
                    color: #fff",
                    {text}
                }
            }
        }
    }
}

pub fn make_toast(text: impl AsRef<str>, show_time: Option<u32>) {
    TOAST_BOX_TEXT.write().push(ToastOption {
        text: text.as_ref().to_string(),
        show_time: show_time.unwrap_or(2500),
    });
}
