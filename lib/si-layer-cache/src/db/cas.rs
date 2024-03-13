use std::sync::Arc;

use si_events::{Actor, CasPk, CasValue, ContentHash, Tenancy, WebEvent};

use crate::{
    error::LayerDbResult,
    event::{LayeredEvent, LayeredEventKind},
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterStatusReader},
};

pub const DBNAME: &str = "cas";
pub const PARTITION_KEY: &str = "cas";

#[derive(Debug, Clone)]
pub struct CasDb {
    pub cache: LayerCache<CasPk, Arc<CasValue>>,
    persister_client: PersisterClient,
}

impl CasDb {
    pub fn new(cache: LayerCache<CasPk, Arc<CasValue>>, persister_client: PersisterClient) -> Self {
        CasDb {
            cache,
            persister_client,
        }
    }

    pub async fn write(
        &self,
        value: Arc<CasValue>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<(CasPk, PersisterStatusReader)> {
        let postcard_value = postcard::to_stdvec(&value)?;
        let key = CasPk::new(ContentHash::new(&postcard_value));
        self.cache.insert(key, value).await;

        let event = LayeredEvent::new(
            LayeredEventKind::CasInsertion,
            Arc::new(DBNAME.to_string()),
            Arc::new(key.as_bytes().to_vec()),
            Arc::new(postcard_value),
            Arc::new("cas".to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;

        Ok((key, reader))
    }

    pub async fn read(&self, key: &CasPk) -> LayerDbResult<Option<Arc<CasValue>>> {
        self.cache.get(key).await
    }
}
