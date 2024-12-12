use std::collections::HashMap;
use std::sync::Arc;

use si_events::{Actor, EncryptedSecretKey, Tenancy, WebEvent};

use crate::hybrid_cache::CacheItem;
use crate::{
    error::LayerDbResult,
    event::{LayeredEvent, LayeredEventKind},
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterStatusReader},
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

    pub fn write(
        &self,
        key: EncryptedSecretKey,
        value: CacheItem,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<PersisterStatusReader> {
        let (postcard_value, size_hint) = serialize::to_vec(&value)?;

        let cache_key: Arc<str> = key.to_string().into();

        self.cache
            .insert(cache_key.clone(), value.clone(), size_hint);

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

    pub async fn read(&self, key: &EncryptedSecretKey) -> LayerDbResult<Option<CacheItem>> {
        self.cache.get(key.to_string().into()).await
    }

    pub async fn read_many(
        &self,
        keys: &[EncryptedSecretKey],
    ) -> LayerDbResult<HashMap<EncryptedSecretKey, CacheItem>> {
        self.cache.get_bulk(keys).await
    }
}
