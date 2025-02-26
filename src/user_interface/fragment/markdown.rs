use crate::util::sleep::sleep;
use dioxus::core_macro::component;
use dioxus::dioxus_core::Element;
use dioxus::prelude::*;
use dioxus::warnings::Warning;
use dioxus_signals::Signal;
use std::borrow::Cow;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[component]
pub fn MarkdownFragment(md_text: String) -> Element {
    let mut id = use_memo(Uuid::new_v4);

    let mut md_text_newest = use_memo(use_reactive!(|md_text| { Arc::new(md_text) }));
    let answer = use_memo(move || markdown::to_html(md_text_newest.read().as_str()));
    let mut md_text = use_hook(|| Arc::new(Mutex::new(Arc::clone(&*md_text_newest.peek()))));

    let _set_height_daemon = use_resource(move || {
        to_owned![md_text];
        async move {
            let id = id.read().to_string();
            let md_text_newest = Arc::clone(&*md_text_newest.read());
            let md_text_old = {
                let mut lock = md_text.lock().unwrap();
                let rv = Arc::clone(&*lock);
                *lock = md_text_newest.clone();
                rv
            };

            //解决切换会话时元素复用产生的错误高度
            {
                let need_reset_height = md_text_newest.len() < md_text_old.len()
                    || (md_text_newest.as_str().starts_with(md_text_old.as_str()) == false);

                if need_reset_height {
                    sleep(100).await;
                    document::eval(&format!(
                        "let iframe=document.getElementById('{id}');if(iframe){{iframe.style.height='1px';}};",
                    ));
                }
            }

            let set_height = format!(
                r#"let iframe = document.getElementById('{id}');
                if(iframe && iframe?.contentWindow?.document?.documentElement && iframe.style.height != iframe.contentWindow.document.documentElement.scrollHeight + 'px'){{
                    iframe.style.height = iframe.contentWindow.document.documentElement.scrollHeight + 'px';
                }};"#,
            );
            for _ in 0..20 {
                sleep(100).await;
                document::eval(&set_height);
            }
            loop {
                sleep(2000).await;
                document::eval(&set_height);
            }
        }
    });

    rsx! {
        iframe {
            id: id.read().to_string(),
            class: "w-full",
            style: "height:1px",
            srcdoc: answer,
        }
    }
}
