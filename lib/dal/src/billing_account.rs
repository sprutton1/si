use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    pk, schema::variant::SchemaVariantError, standard_model_accessor_ro, DalContext, HistoryActor,
    HistoryEvent, HistoryEventError, KeyPair, KeyPairError, NodeError, SchemaError, Tenancy,
    Timestamp, TransactionsError, User, UserError, Workspace, WorkspaceError,
};

const BILLING_ACCOUNT_GET_BY_NAME: &str = include_str!("queries/billing_account/get_by_name.sql");
const BILLING_ACCOUNT_GET_BY_PK: &str = include_str!("queries/billing_account/get_by_pk.sql");
const BILLING_ACCOUNT_GET_DEFAULTS: &str = include_str!("queries/billing_account/get_defaults.sql");

#[derive(Error, Debug)]
pub enum BillingAccountError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
}

pub type BillingAccountResult<T> = Result<T, BillingAccountError>;

pk!(BillingAccountPk);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BillingAccount {
    pk: BillingAccountPk,
    name: String,
    description: Option<String>,
    #[serde(flatten)]
    timestamp: Timestamp,
    visibility_deleted_at: Option<DateTime<Utc>>,
}

impl BillingAccount {
    pub fn pk(&self) -> &BillingAccountPk {
        &self.pk
    }

    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        description: Option<&String>,
    ) -> BillingAccountResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM billing_account_create_v1($1, $2)",
                &[&name, &description],
            )
            .await?;

        // Inlined `finish_create_from_row`

        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;

        // HistoryEvent won't be accessible by any tenancy (null tenancy_workspace_pk)
        let _history_event = HistoryEvent::new(
            ctx,
            "billing_account.create".to_owned(),
            "Billing Account created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor_ro!(name, String);
    standard_model_accessor_ro!(description, Option<String>);

    pub async fn signup(
        ctx: &mut DalContext,
        billing_account_name: impl AsRef<str>,
        user_name: impl AsRef<str>,
        user_email: impl AsRef<str>,
        user_password: impl AsRef<str>,
    ) -> BillingAccountResult<BillingAccountSignup> {
        let billing_account = BillingAccount::new(&*ctx, billing_account_name, None).await?;
        let workspace = Workspace::new(ctx, "default", *billing_account.pk()).await?;

        let key_pair = KeyPair::new(&*ctx, "default").await?;

        let user = User::new(&*ctx, &user_name, &user_email, &user_password).await?;
        let user_history_actor = HistoryActor::User(user.pk());
        ctx.update_history_actor(user_history_actor);

        ctx.import_builtins().await?;

        Ok(BillingAccountSignup {
            billing_account,
            key_pair,
            user,
            workspace,
        })
    }

    pub async fn get_by_pk(
        ctx: &DalContext,
        pk: &BillingAccountPk,
    ) -> BillingAccountResult<BillingAccount> {
        let row = ctx
            .txns()
            .pg()
            .query_one(BILLING_ACCOUNT_GET_BY_PK, &[&pk])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        Ok(serde_json::from_value(json)?)
    }

    pub async fn find_by_name(
        ctx: &DalContext,
        name: impl AsRef<str>,
    ) -> BillingAccountResult<Option<BillingAccount>> {
        let name = name.as_ref();
        let maybe_row = ctx
            .txns()
            .pg()
            .query_opt(BILLING_ACCOUNT_GET_BY_NAME, &[&name])
            .await?;

        let result = match maybe_row {
            Some(row) => {
                let json: serde_json::Value = row.try_get("object")?;
                Some(serde_json::from_value(json)?)
            }
            None => None,
        };
        Ok(result)
    }

    pub async fn get_defaults(
        ctx: &DalContext,
        pk: &BillingAccountPk,
    ) -> BillingAccountResult<BillingAccountDefaults> {
        let row = ctx
            .txns()
            .pg()
            .query_one(BILLING_ACCOUNT_GET_DEFAULTS, &[&pk])
            .await?;
        let workspace_json: serde_json::Value = row.try_get("workspace")?;
        let workspace: Workspace = serde_json::from_value(workspace_json)?;

        let mut workspace_ctx = ctx.clone();
        workspace_ctx.update_tenancy(Tenancy::new(*workspace.pk()));

        let result = BillingAccountDefaults { workspace };
        Ok(result)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BillingAccountSignup {
    pub billing_account: BillingAccount,
    pub key_pair: KeyPair,
    pub user: User,
    pub workspace: Workspace,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BillingAccountDefaults {
    pub workspace: Workspace,
}
