#![allow(non_snake_case)]

mod env;
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

fn main() {
    dioxus::launch(app);
}
