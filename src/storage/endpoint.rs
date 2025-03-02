use crate::storage::data_store::DataStore;
use serde::{Deserialize, Serialize};

pub static MODEL_ENDPOINT_STORE: DataStore<ModelEndpoint> = DataStore::new("endpoints");

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct ModelEndpoint {
    pub base: String,
    pub key: String,
    pub model: String,
}

impl ModelEndpoint {
    pub fn is_valid(&self) -> bool {
        false == (self.model.is_empty() || self.base.is_empty() || self.key.is_empty())
    }
}

pub async fn get_endpoint_store() -> &'static DataStore<ModelEndpoint> {
    &MODEL_ENDPOINT_STORE
}
