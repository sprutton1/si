use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use si_events::{Actor, CasValue, ContentHash, Tenancy, WebEvent};

use crate::hybrid_cache::{CacheItem, CacheItemSpec};
use crate::LayerDbError;
use crate::{
    error::LayerDbResult,
    event::{LayeredEvent, LayeredEventKind},
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterStatusReader},
};

use super::serialize;

pub const DBNAME: &str = "cas";
pub const CACHE_NAME: &str = "cas";
pub const PARTITION_KEY: &str = "cas";

#[derive(Debug, Clone)]
pub struct CasDb {
    pub cache: Arc<LayerCache>,
    persister_client: PersisterClient,
}

#[typetag::serde]
impl CacheItemSpec for CasValue {}

impl CasDb {
    pub fn new(cache: Arc<LayerCache>, persister_client: PersisterClient) -> Self {
        CasDb {
            cache,
            persister_client,
        }
    }

    pub fn write(
        &self,
        value: CacheItem,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<(ContentHash, PersisterStatusReader)> {
        let (postcard_value, size_hint) = serialize::to_vec(&value)?;
        let key = ContentHash::new(&postcard_value);
        let cache_key: Arc<str> = key.to_string().into();

        self.cache
            .insert(cache_key.clone(), value.clone(), size_hint);

        let event = LayeredEvent::new(
            LayeredEventKind::CasInsertion,
            Arc::new(DBNAME.to_string()),
            cache_key,
            Arc::new(postcard_value),
            Arc::new("cas".to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;

        Ok((key, reader))
    }

    pub async fn read(&self, key: &ContentHash) -> LayerDbResult<Option<Arc<CasValue>>> {
        Ok(match self.cache.get(key.to_string().into()).await {
            Ok(Some(value)) => Some(value.downcast_arc::<CasValue>().unwrap()),
            _ => None,
        })
    }

    /// We often need to extract the value from the arc by cloning it (although
    /// this should be avoided for large values). This will do that, and also
    /// helpfully convert the value to the type we want to deal with
    pub async fn try_read_as<T>(&self, key: &ContentHash) -> LayerDbResult<Option<T>>
    where
        CasValue: TryInto<T>,
        <CasValue as TryInto<T>>::Error: Display,
    {
        Ok(match self.read(key).await? {
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

    pub async fn read_many(
        &self,
        keys: &[ContentHash],
    ) -> LayerDbResult<HashMap<ContentHash, Arc<CasValue>>> {
        self.try_read_many_as::<CasValue>(keys).await
    }

    pub async fn try_read_many_as<T>(
        &self,
        keys: &[ContentHash],
    ) -> LayerDbResult<HashMap<ContentHash, Arc<T>>>
    where
        T: 'static + CacheItemSpec + Clone,
    {
        let mut result = HashMap::new();
        for (key, arc_v) in self.cache.get_bulk(keys).await? {
            let value = arc_v.downcast_arc::<T>().unwrap();
            result.insert(key, value);
        }

        Ok(result)
    }
}
