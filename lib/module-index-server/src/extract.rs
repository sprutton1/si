use std::fmt;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts, Json};
use hyper::StatusCode;
use s3::{Bucket as S3Bucket, Region as AwsRegion};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use si_jwt_public_key::SiJwtClaims;

use super::app_state::AppState;

pub struct ExtractedS3Bucket(pub S3Bucket);

#[async_trait]
impl FromRequestParts<AppState> for ExtractedS3Bucket {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let region = match state.s3_config().region.parse::<AwsRegion>() {
            Ok(region) => region,
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": {
                            "message": err.to_string(),
                            "code": 42,
                            "statusCode": 500
                        }
                    })),
                ))
            }
        };

        let bucket = S3Bucket::new(&state.s3_config().bucket, region, state.aws_creds().clone())
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": {
                            "message": err.to_string(),
                            "code": 42,
                            "statusCode": 500
                        }
                    })),
                )
            })?;
        Ok(ExtractedS3Bucket(bucket))
    }
}

pub struct DbConnection(pub DatabaseTransaction);

#[async_trait]
impl FromRequestParts<AppState> for DbConnection {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match state.pg_pool().begin().await {
            Ok(txn) => Ok(DbConnection(txn)),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "message": err.to_string(),
                        "code": 42,
                        "statusCode": 500
                    }
                })),
            )),
        }
    }
}

pub struct Authorization {
    pub user_claim: SiJwtClaims,
    pub auth_token: String,
}

#[async_trait]
impl FromRequestParts<AppState> for Authorization {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jwt_public_signing_key = state.jwt_public_signing_key();

        let headers = &parts.headers;
        let authorization_header_value = headers
            .get("Authorization")
            .ok_or_else(unauthorized_error)?;
        let auth_token = authorization_header_value
            .to_str()
            .map_err(internal_error)?;
        let user_claim =
            si_jwt_public_key::validate_bearer_token(jwt_public_signing_key.clone(), &auth_token)
                .await
                .map_err(|_| unauthorized_error())?
                .custom;

        Ok(Self {
            user_claim: user_claim.into(),
            auth_token: auth_token.into(),
        })
    }
}

fn internal_error(message: impl fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::INTERNAL_SERVER_ERROR;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message.to_string(),
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}

fn unauthorized_error() -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::UNAUTHORIZED;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": "unauthorized",
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}
