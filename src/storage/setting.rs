use crate::storage::Store;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub static SETTING_STORE: Store<Setting> = Store::new("setting");

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Setting {}

impl Default for Setting {
    fn default() -> Self {
        Self {}
    }
}

pub async fn get_session_store() -> &'static Store<Setting> {
    &SETTING_STORE
}
