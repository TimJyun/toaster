#![allow(non_snake_case)]

mod constant;
mod error;
mod i18n;
mod imgs;
mod native;
mod openai;
mod sound;
pub mod storage;
mod text;
mod tts;
mod user_interface;
mod util;

use crate::user_interface::app::app;
use dioxus_signals::Readable;

fn main() {
    #[cfg(feature = "web")]
    {
        use js_sys::wasm_bindgen::JsCast;
        use js_sys::wasm_bindgen::closure::Closure;
        if let Some(window) = web_sys::window() {
            let closure: Closure<dyn Fn(web_sys::BeforeUnloadEvent)> =
                Closure::new(move |event: web_sys::BeforeUnloadEvent| {
                    use crate::openai::INFERENCING;
                    if INFERENCING.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                        event.prevent_default();
                        event.set_return_value("AI正在推理中，现在退出将会丢失部分进度");
                    }
                });
            let _ = window
                .add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }

    dioxus::launch(app);
}
