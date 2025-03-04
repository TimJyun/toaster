use dioxus_signals::{GlobalSignal, MappedSignal, Readable, Signal};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

pub struct ConfigStore<T: Serialize + DeserializeOwned + Default + Clone + 'static> {
    config_name: &'static str,
    signal: GlobalSignal<T>,
    initialized: AtomicBool,
}

impl<T: Serialize + DeserializeOwned + Default + Clone + 'static> ConfigStore<T> {
    pub const fn new(config_name: &'static str) -> Self {
        Self {
            config_name,
            signal: Signal::global(Default::default),
            initialized: AtomicBool::new(false),
        }
    }

    fn get_from_storage(&self) -> T {
        let config_name = self.config_name;
        #[cfg(feature = "web")]
        {
            use gloo_storage::Storage;
            gloo_storage::LocalStorage::get(config_name).unwrap_or_default()
        }
        #[cfg(feature = "native")]
        {
            use crate::constant::TOASTER_DATA_PATH;
            std::fs::read_to_string(format!("{TOASTER_DATA_PATH}/{config_name}"))
                .ok()
                .map(|str| serde_json::from_str::<T>(&str).ok())
                .flatten()
                .unwrap_or_default()
        }
    }

    fn save_to_storage(&self, value: &T) -> Result<(), anyhow::Error> {
        let config_name = self.config_name;

        #[cfg(feature = "web")]
        {
            use gloo_storage::Storage;
            gloo_storage::LocalStorage::set(config_name, value)?;
        }
        #[cfg(feature = "native")]
        {
            use crate::constant::TOASTER_DATA_PATH;
            use crate::constant::TOASTER_TMP_DATA_PATH;
            use std::fs;
            let json_str = serde_json::to_string(value)?;

            let bytes = json_str.as_bytes();
            let uuid = Uuid::new_v4();
            let tmp_path = format!("{TOASTER_TMP_DATA_PATH}/{uuid}.json");
            let mut file = fs::File::create(&tmp_path)?;
            file.write_all(bytes)?;
            fs::rename(&tmp_path, format!("{TOASTER_DATA_PATH}/{config_name}.json"))?;
        }

        Ok(())
    }

    pub fn get(&self) -> MappedSignal<T> {
        if let Ok(false) =
            self.initialized
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        {
            *self.signal.write() = self.get_from_storage()
        }
        let b = self.signal.signal().map(|a| a);
        b
    }

    pub fn set(&self, value: T) -> Result<(), anyhow::Error> {
        self.save_to_storage(&value)?;
        *self.signal.write() = value;
        Ok(())
    }
}
