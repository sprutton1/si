use axum::Json;
use dal::{User, Workspace};
use serde::{Deserialize, Serialize};

use super::{SessionError, SessionResult};
use crate::extract::{AccessBuilder, Authorization, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuthenticationResponse {
    pub user: User,
    pub workspace: Workspace,
}

pub async fn restore_authentication(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Authorization(claim): Authorization,
) -> SessionResult<Json<RestoreAuthenticationResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let workspace = Workspace::get_by_pk(&ctx, &claim.workspace_id)
        .await?
        .ok_or(SessionError::InvalidWorkspace(claim.workspace_id))?;

    let user = User::get_by_pk(&ctx, claim.user_id)
        .await?
        .ok_or(SessionError::InvalidUser(claim.user_id))?;

    let reply = RestoreAuthenticationResponse { user, workspace };

    Ok(Json(reply))
}
