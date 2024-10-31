use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};

use serde::{de::DeserializeOwned, Serialize};
use si_events::{Actor, EncryptedSecretKey, Tenancy, WebEvent};

use crate::{
    error::LayerDbResult,
    event::{LayeredEvent, LayeredEventKind},
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterStatusReader},
    LayerDbError,
};

use super::serialize;

const KEYWORD_SINGULAR: &str = "encrypted_secret";
const KEYWORD_PLURAL: &str = "encrypted_secrets";

pub const PARTITION_KEY: &str = KEYWORD_PLURAL;
pub const DBNAME: &str = KEYWORD_PLURAL;
pub const CACHE_NAME: &str = KEYWORD_PLURAL;
pub const SORT_KEY: &str = KEYWORD_SINGULAR;

#[derive(Debug, Clone)]
pub struct EncryptedSecretDb {
    pub cache: Arc<LayerCache>,
    persister_client: PersisterClient,
}

impl EncryptedSecretDb {
    pub fn new(cache: Arc<LayerCache>, persister_client: PersisterClient) -> Self {
        EncryptedSecretDb {
            cache,
            persister_client,
        }
    }

    pub fn write<V>(
        &self,
        key: EncryptedSecretKey,
        value: Arc<V>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<PersisterStatusReader>
    where
        V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
    {
        let postcard_value = serialize::to_vec(&value)?;

        let cache_key: Arc<str> = key.to_string().into();

        self.cache.insert(cache_key.clone(), value.clone());

        let event = LayeredEvent::new(
            LayeredEventKind::EncryptedSecretInsertion,
            Arc::new(DBNAME.to_string()),
            cache_key,
            Arc::new(postcard_value),
            Arc::new(SORT_KEY.to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;

        Ok(reader)
    }

    pub async fn read<V>(&self, key: &EncryptedSecretKey) -> LayerDbResult<Option<Arc<V>>>
    where
        V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
    {
        self.cache.get(key.to_string().into()).await
    }

    /// We often need to extract the value from the arc by cloning it (although
    /// this should be avoided for large values). This will do that, and also
    /// helpfully convert the value to the type we want to deal with
    pub async fn try_read_as<T, V>(&self, key: &EncryptedSecretKey) -> LayerDbResult<Option<T>>
    where
        V: TryInto<T>,
        <V as TryInto<T>>::Error: Display,
        V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
    {
        Ok(match self.read::<V>(key).await? {
            None => None,
            Some(arc_v) => Some(
                arc_v
                    .as_ref()
                    .clone()
                    .try_into()
                    .map_err(|err| LayerDbError::ContentConversion(err.to_string()))?,
            ),
        })
    }

    pub async fn read_many<V>(
        &self,
        keys: &[EncryptedSecretKey],
    ) -> LayerDbResult<HashMap<EncryptedSecretKey, Arc<V>>>
    where
        V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
    {
        self.cache.get_bulk(keys).await
    }

    pub async fn try_read_many_as<T, V>(
        &self,
        keys: &[EncryptedSecretKey],
    ) -> LayerDbResult<HashMap<EncryptedSecretKey, T>>
    where
        V: TryInto<T> + Clone + DeserializeOwned + Serialize + Send + Sync + 'static + AsRef<T>,
        <V as TryInto<T>>::Error: Display,
        T: for<'a> std::convert::From<&'a T>,
    {
        let mut result = HashMap::new();
        for (key, arc_v) in self.cache.get_bulk::<EncryptedSecretKey, V>(keys).await? {
            result.insert(
                key,
                arc_v
                    .as_ref()
                    .clone()
                    .try_into()
                    .map_err(|_| LayerDbError::ContentConversion("shit".to_string()))?,
            );
        }

        Ok(result)
    }
}
