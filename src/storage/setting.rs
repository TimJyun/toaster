use crate::storage::config_store::ConfigStore;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub static SETTING_STORE: ConfigStore<Setting> = ConfigStore::new("sessions");

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Setting {
    pub initialized: bool,
    pub current_version: String,
}

impl Default for Setting {
    fn default() -> Self {
        Self {
            initialized: false,
            current_version: String::new(),
        }
    }
}

pub fn get_setting() -> &'static ConfigStore<Setting> {
    &SETTING_STORE
}
