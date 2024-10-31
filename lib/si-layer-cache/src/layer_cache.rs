use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};

use serde::de::DeserializeOwned;
use si_data_pg::PgPool;
use si_runtime::DedicatedExecutor;
use telemetry::prelude::*;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::db::serialize;
use crate::error::LayerDbResult;
use crate::hybrid_cache::{Cache, CacheConfig};
use crate::pg::PgLayer;
use crate::LayerDbError;

#[derive(Debug, Clone)]
pub struct LayerCache {
    cache: Cache,
    name: String,
    pg: PgLayer,
    #[allow(dead_code)]
    compute_executor: DedicatedExecutor,
}

impl LayerCache {
    pub async fn new(
        name: &str,
        pg_pool: PgPool,
        cache_config: CacheConfig,
        #[allow(dead_code)] compute_executor: DedicatedExecutor,
        tracker: TaskTracker,
        token: CancellationToken,
    ) -> LayerDbResult<Arc<Self>> {
        let cache = Cache::new(cache_config).await?;

        let pg = PgLayer::new(pg_pool.clone(), name);

        let lc: Arc<LayerCache> = LayerCache {
            cache,
            name: name.to_string(),
            pg,
            compute_executor,
        }
        .into();

        tracker.spawn(lc.clone().shutdown_handler(token.clone()));
        Ok(lc)
    }

    async fn shutdown_handler(self: Arc<Self>, token: CancellationToken) -> LayerDbResult<()> {
        token.cancelled().await;
        debug!("shutting down layer cache {}", self.name);
        // foyer will wait on all outstanding flush and reclaim threads here
        self.cache().close().await?;
        Ok(())
    }

    pub async fn get<V>(&self, key: Arc<str>) -> LayerDbResult<Option<V>>
    where
        V: Clone + DeserializeOwned + Send + Sync + 'static,
    {
        Ok(match self.cache.get(&key).await {
            Some(memory_value) => Some(memory_value),

            None => match self.pg.get(&key).await? {
                Some(value) => {
                    let deserialized: V = serialize::from_bytes(&value)?;

                    self.cache.insert(key.clone(), deserialized.clone());

                    Some(deserialized)
                }
                None => None,
            },
        })
    }

    #[instrument(
        name = "layer_cache.get_bytes_from_durable_storage",
        level = "debug",
        skip_all,
        fields(
            si.layer_cache.key = key.as_ref(),
        ),
    )]
    pub async fn get_bytes_from_durable_storage(
        &self,
        key: Arc<str>,
    ) -> LayerDbResult<Option<Vec<u8>>> {
        self.pg.get(&key).await
    }

    pub async fn get_bulk<K, V>(&self, keys: &[K]) -> LayerDbResult<HashMap<K, V>>
    where
        K: Clone + Display + Eq + Hash + FromStr,
        <K as FromStr>::Err: Display,
        V: Clone + DeserializeOwned + Send + Sync + 'static,
    {
        let mut found_keys = HashMap::new();
        let mut not_found: Vec<Arc<str>> = vec![];

        for key in keys {
            let key_str: Arc<str> = key.to_string().into();
            if let Some(found) = match self.cache.get(&key_str).await {
                Some(value) => Some(value),
                None => {
                    not_found.push(key_str.clone());
                    None
                }
            } {
                found_keys.insert(key.clone(), found);
            }
        }

        if !not_found.is_empty() {
            if let Some(pg_found) = self.pg.get_many(&not_found).await? {
                for (k, v) in pg_found {
                    let deserialized: V = serialize::from_bytes(&v)?;
                    self.cache.insert(k.clone().into(), deserialized.clone());
                    found_keys.insert(
                        K::from_str(&k).map_err(|err| {
                            LayerDbError::CouldNotConvertToKeyFromString(err.to_string())
                        })?,
                        deserialized,
                    );
                }
            }
        }

        Ok(found_keys)
    }

    pub async fn deserialize_memory_value<V>(&self, bytes: Arc<Vec<u8>>) -> LayerDbResult<V>
    where
        V: Clone + DeserializeOwned + Send + Sync + 'static,
    {
        serialize::from_bytes_async(&bytes)
            .await
            .map_err(Into::into)
    }

    pub fn cache(&self) -> Cache {
        self.cache.clone()
    }

    pub fn pg(&self) -> PgLayer {
        self.pg.clone()
    }

    pub fn remove_from_memory(&self, key: &str) {
        self.cache.remove(key);
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains(key)
    }

    pub fn insert<V>(&self, key: Arc<str>, value: V)
    where
        V: Clone + DeserializeOwned + Send + Sync + 'static,
    {
        if !self.cache.contains(&key) {
            self.cache.insert(key, value);
        }
    }

    pub fn insert_from_cache_updates(&self, key: Arc<str>, serialize_value: Vec<u8>) {
        self.cache
            .insert_raw_bytes(key.clone(), serialize_value.clone());
    }

    pub fn insert_or_update<V: Clone + DeserializeOwned + Send + Sync + 'static>(
        &self,
        key: Arc<str>,
        value: V,
    ) {
        self.cache.insert(key, value);
    }

    pub fn insert_or_update_from_cache_updates(&self, key: Arc<str>, serialize_value: Vec<u8>) {
        self.insert_from_cache_updates(key, serialize_value)
    }

    pub fn evict_from_cache_updates(&self, key: Arc<str>) {
        self.cache.remove(&key);
    }
}
