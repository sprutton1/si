use axum::extract::OriginalUri;
use axum::Json;
use dal::edge::EdgeKind;
use dal::{
    job::definition::DependentValuesUpdate, node::NodeId, socket::SocketId, AttributeReadContext,
    AttributeValue, Connection, ExternalProvider, Node, Socket, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::{DiagramError, DiagramResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub from_node_id: NodeId,
    pub from_socket_id: SocketId,
    pub to_node_id: NodeId,
    pub to_socket_id: SocketId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionResponse {
    pub connection: Connection,
}

/// Create a [`Connection`](dal::Connection) with a _to_ [`Socket`](dal::Socket) and
/// [`Node`](dal::Node) and a _from_ [`Socket`](dal::Socket) and [`Node`](dal::Node).
pub async fn create_connection(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateConnectionRequest>,
) -> DiagramResult<Json<CreateConnectionResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let connection = Connection::new(
        &ctx,
        request.from_node_id,
        request.from_socket_id,
        request.to_node_id,
        request.to_socket_id,
        EdgeKind::Configuration,
    )
    .await?;

    let from_component = Node::get_by_id(&ctx, &request.from_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(request.from_node_id))?
        .component(&ctx)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let from_component_schema = from_component
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let from_socket = Socket::get_by_id(&ctx, &request.from_socket_id)
        .await?
        .ok_or(DiagramError::SocketNotFound)?;

    let to_component = Node::get_by_id(&ctx, &request.to_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(request.to_node_id))?
        .component(&ctx)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let to_component_schema = to_component
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let to_socket = Socket::get_by_id(&ctx, &request.to_socket_id)
        .await?
        .ok_or(DiagramError::SocketNotFound)?;

    let from_socket_external_provider =
        ExternalProvider::find_for_socket(&ctx, request.from_socket_id)
            .await?
            .ok_or(DiagramError::ExternalProviderNotFoundForSocket(
                request.from_socket_id,
            ))?;

    let attribute_value_context = AttributeReadContext {
        external_provider_id: Some(*from_socket_external_provider.id()),
        component_id: Some(*from_component.id()),
        ..Default::default()
    };
    let attribute_value = AttributeValue::find_for_context(&ctx, attribute_value_context)
        .await?
        .ok_or(DiagramError::AttributeValueNotFoundForContext(
            attribute_value_context,
        ))?;

    ctx.enqueue_job(DependentValuesUpdate::new(
        ctx.access_builder(),
        *ctx.visibility(),
        vec![*attribute_value.id()],
    ))
    .await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "connection_created",
        serde_json::json!({
                    "from_node_id": request.from_node_id,
                    "from_node_schema_name": &from_component_schema.name(),
                    "from_socket_id": request.from_socket_id,
                    "from_socket_name": &from_socket.name(),
                    "to_node_id": request.to_node_id,
                    "to_node_schema_name": &to_component_schema.name(),
                    "to_socket_id": request.to_socket_id,
                    "to_socket_name":  &to_socket.name(),
        }),
    );

    ctx.commit().await?;

    Ok(Json(CreateConnectionResponse { connection }))
}
