use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::func::backend::{FuncBackend, FuncBackendResult};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendResourcePayloadToValueArgs {
    pub payload: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendResourcePayloadToValue {
    args: FuncBackendResourcePayloadToValueArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendResourcePayloadToValue {
    type Args = FuncBackendResourcePayloadToValueArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let value = serde_json::to_value(&self.args.payload)?;
        Ok((Some(value.clone()), Some(value)))
    }
}
