use crate::util::sleep::sleep;
use dioxus::core_macro::component;
use dioxus::dioxus_core::Element;
use dioxus::prelude::*;
use dioxus::warnings::Warning;
use dioxus_signals::Signal;
use std::borrow::Cow;
use std::rc::Rc;
use uuid::Uuid;

#[component]
pub fn MarkdownFragment(md_text: String) -> Element {
    let mut id = use_memo(use_reactive!(|md_text| { Uuid::new_v4() }));
    let answer = use_memo(use_reactive!(|md_text| {
        markdown::to_html(md_text.as_str())
    }));

    use_resource(move || async move {
        let id = id.read().to_string();
        sleep(100).await;
        //解决切换会话时元素复用产生的错误高度
        document::eval(&format!(
            "let iframe=document.getElementById('{id}');if(iframe){{iframe.style.height='1px';}};",
        ));
        loop {
            sleep(100).await;
            document::eval(&format!(
                r#"let iframe=document.getElementById('{id}');if(iframe){{iframe.style.height=iframe.contentWindow.document.documentElement.scrollHeight+'px';}};"#,
            ));
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
