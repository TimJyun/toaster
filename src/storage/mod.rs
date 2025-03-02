use dioxus_signals::Readable;
use opendal::Configurator;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::ops::{Deref, DerefMut};

pub mod config_store;
pub mod data_store;
pub mod endpoint;
pub mod session;
pub mod setting;
