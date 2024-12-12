use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{schema::variant::authoring::VariantAuthoringClient, ChangeSet, Visibility, WsEvent};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;
use si_frontend_types::SchemaVariant as FrontendVariant;

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::{force_change_set_response::ForceChangeSetResponse, variant::SchemaVariantResult},
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantRequest {
    pub name: String,
    pub color: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn create_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CreateVariantRequest>,
) -> SchemaVariantResult<ForceChangeSetResponse<FrontendVariant>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let created_schema_variant = VariantAuthoringClient::create_schema_and_variant(
        &ctx,
        request.name.clone(),
        None::<String>,
        None::<String>,
        "".to_string(),
        request.color.clone(),
    )
    .await?;

    let schema = created_schema_variant.schema(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_variant",
        serde_json::json!({
            "variant_name": request.name.clone(),
            "variant_id": created_schema_variant.id().clone(),
            "schema_id": schema.id(),
        }),
    );

    WsEvent::schema_variant_created(&ctx, schema.id(), created_schema_variant.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.write_audit_log(
        AuditLogKind::CreateSchemaVariant {
            schema_id: schema.id().into(),
            schema_variant_id: created_schema_variant.id().into(),
        },
        created_schema_variant.display_name().to_string(),
    )
    .await?;

    ctx.commit().await?;

    let variant = created_schema_variant
        .into_frontend_type(&ctx, schema.id())
        .await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, variant))
}
