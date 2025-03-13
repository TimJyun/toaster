use crate::util::sleep::sleep;
use dioxus::prelude::*;
use dioxus_signals::{GlobalSignal, Readable, Signal};
use once_cell::sync::OnceCell;
use opendal::Configurator;
use opendal::Operator;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::marker::PhantomData;
use std::mem::{needs_drop, swap};
use std::ops::{Deref, DerefMut};
use std::panic;
use std::sync::{Arc, Mutex};
use tracing::debug;

pub struct DataStore<T: Serialize + DeserializeOwned> {
    db_name: &'static str,
    op: OnceCell<Operator>,
    subscribers: Mutex<BTreeMap<String, HashSet<ReactiveContext>>>,
    phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> DataStore<T> {
    pub const fn new(db_name: &'static str) -> Self {
        Self {
            db_name,
            op: OnceCell::new(),
            subscribers: Mutex::new(BTreeMap::new()),
            phantom: PhantomData,
        }
    }

    fn get_operator(&self) -> &Operator {
        let db_name = self.db_name;
        self.op.get_or_init(|| {
            #[cfg(all(feature = "web", target_family = "wasm"))]
            {
                use opendal_indexeddb::config::IndexeddbConfig;
                let session_store_config = IndexeddbConfig {
                    db_name: None,
                    object_store_name: Some(db_name.to_string()),
                    root: None,
                };
                let builder = session_store_config.into_builder();
                let op = Operator::new(builder).unwrap().finish();
                return op;
            }
            #[cfg(any(feature = "mobile", feature = "desktop"))]
            {
                use crate::constant::TOASTER_DATA_PATH;
                use crate::constant::TOASTER_TMP_DATA_PATH;
                use opendal::Configurator;
                use opendal::services::FsConfig;
                let mut fs_config = FsConfig::default();

                fs_config.root = Some(format!("{TOASTER_DATA_PATH}/{}", db_name));
                fs_config.atomic_write_dir = Some(format!("{TOASTER_TMP_DATA_PATH}/{}", db_name));

                let builder = fs_config.into_builder();
                let op = Operator::new(builder).unwrap().finish();
                return op;
            }
        })
    }

    pub async fn get(&self, key: impl AsRef<str>) -> Result<T, anyhow::Error> {
        if let Some(rc) = ReactiveContext::current() {
            self.subscribers
                .lock()
                .unwrap()
                .entry(key.as_ref().to_string())
                .or_default()
                .insert(rc);
        } else {
            debug!("not ReactiveContext found");
        }

        let buff = self.get_operator().read(key.as_ref()).await?.to_vec();
        let value = ciborium::from_reader::<T, _>(buff.as_slice())?;
        Ok(value)
    }

    pub async fn set(&self, key: impl AsRef<str>, value: &T) -> Result<(), anyhow::Error> {
        let mut buff = Vec::<u8>::new();
        ciborium::into_writer(&value, &mut buff)?;
        self.get_operator().write(key.as_ref(), buff).await?;

        // update components
        let mut subscribers = self.subscribers.lock().unwrap();
        let list_subscribers = subscribers.deref_mut().remove("");
        let key_subscribers = subscribers.deref_mut().remove(key.as_ref());
        drop(subscribers);
        let subscribers = if let Some(mut list_subscribers) = list_subscribers {
            list_subscribers.extend(key_subscribers.unwrap_or_default());
            list_subscribers
        } else {
            key_subscribers.unwrap_or_default()
        };

        for subscriber in subscribers.into_iter() {
            subscriber.mark_dirty();
        }

        Ok(())
    }

    pub async fn delete(&self, key: impl AsRef<str>) -> Result<(), anyhow::Error> {
        self.get_operator().delete(key.as_ref()).await?;

        // update components
        let mut subscribers = self.subscribers.lock().unwrap();
        let list_subscribers = subscribers.deref_mut().remove("");
        let key_subscribers = subscribers.deref_mut().remove(key.as_ref());
        drop(subscribers);
        let subscribers = if let Some(mut list_subscribers) = list_subscribers {
            list_subscribers.extend(key_subscribers.unwrap_or_default());
            list_subscribers
        } else {
            key_subscribers.unwrap_or_default()
        };

        for subscriber in subscribers.into_iter() {
            subscriber.mark_dirty();
        }

        Ok(())
    }

    pub async fn list(&self) -> Result<BTreeSet<String>, anyhow::Error> {
        if let Some(rc) = ReactiveContext::current() {
            self.subscribers
                .lock()
                .unwrap()
                .entry(String::new())
                .or_default()
                .insert(rc);
        } else {
            debug!("not ReactiveContext found");
        }

        Ok(self
            .get_operator()
            .list("")
            .await?
            .into_iter()
            .map(|e| e.name().to_string())
            .collect())
    }
}
