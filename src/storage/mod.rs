use async_once_cell::OnceCell;
use dioxus_signals::{GlobalSignal, Readable, Signal};
use opendal::{Configurator, Operator};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub mod endpoint;
pub mod session;
mod setting;

pub struct Store<T: Serialize + DeserializeOwned> {
    db_name: &'static str,
    op: OnceCell<Operator>,
    signal: GlobalSignal<()>,
    phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> Store<T> {
    pub const fn new(db_name: &'static str) -> Self {
        Self {
            db_name,
            op: OnceCell::new(),
            signal: Signal::global(|| ()),
            phantom: PhantomData,
        }
    }

    async fn get_operator(&self) -> &Operator {
        let db_name = self.db_name;
        self.op
            .get_or_init(async move {
                #[cfg(all(feature = "web", target_family = "wasm"))]
                {
                    use opendal_indexeddb::config::IndexeddbConfig;
                    let session_store_config = IndexeddbConfig {
                        db_name: Some(db_name.to_string()),
                        object_store_name: Some(db_name.to_string()),
                        root: None,
                    };
                    let builder = session_store_config.into_builder();
                    let op = Operator::new(builder).unwrap().finish();
                    return op;
                }
                #[cfg(any(feature = "mobile", feature = "desktop"))]
                {
                    use opendal::services::FsConfig;
                    let mut fs_config = FsConfig::default();

                    //todo 标准化data目录
                    fs_config.root = Some(format!("data/root/{}", db_name));
                    fs_config.atomic_write_dir = Some(format!("data/tmp/{}", db_name));

                    let builder = fs_config.into_builder();
                    let op = Operator::new(builder).unwrap().finish();
                    return op;
                }
            })
            .await
    }

    pub async fn get(&self, key: impl AsRef<str>) -> Result<T, anyhow::Error> {
        self.signal.read().deref();
        let buff = self.get_operator().await.read(key.as_ref()).await?.to_vec();
        let session = ciborium::from_reader::<T, _>(buff.as_slice())?;
        Ok(session)
    }

    pub async fn set(&self, key: impl AsRef<str>, value: &T) -> Result<(), anyhow::Error> {
        *self.signal.write() = ();
        let mut buff = Vec::<u8>::new();
        ciborium::into_writer(&value, &mut buff)?;
        self.get_operator().await.write(key.as_ref(), buff).await?;

        Ok(())
    }

    pub async fn delete(&self, key: impl AsRef<str>) -> Result<(), anyhow::Error> {
        *self.signal.write() = ();
        Ok(self.get_operator().await.delete(key.as_ref()).await?)
    }

    pub async fn list(&self) -> Result<BTreeSet<String>, anyhow::Error> {
        self.signal.read().deref();
        Ok(self
            .get_operator()
            .await
            .list("")
            .await?
            .into_iter()
            .map(|e| e.name().to_string())
            .collect())
    }
}
