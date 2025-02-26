use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Setting {
    pub initialized: bool,
}

impl Default for Setting {
    fn default() -> Self {
        Self { initialized: false }
    }
}
