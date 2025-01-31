use auth_client::apis::JwtApi;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use serde::Deserialize;

use crate::{
  core::{
    error::{InternalError, INVALID_ERROR, REQUIRED_ERROR},
    openapi::AUTHORIZATION_HEADER,
  },
  router::RouterState,
  service::auth::jwt_api_client,
};

pub const TOKEN_TYPE_BEARER: &str = "bearer";

pub struct Authorization {
  pub claims: Claims,
}

impl<S> FromRequestParts<S> for Authorization
where
  RouterState: FromRef<S>,
  S: Send + Sync,
{
  type Rejection = InternalError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let state = RouterState::from_ref(_state);

    if let Some(authorization_header_value) = parts.headers.get(AUTHORIZATION_HEADER) {
      let authorization_string = match authorization_header_value.to_str() {
        Ok(authorization_string) => {
          if authorization_string.len() < TOKEN_TYPE_BEARER.len() + 1 {
            return Err(
              InternalError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR),
            );
          }
          &authorization_string[(TOKEN_TYPE_BEARER.len() + 1)..]
        }
        Err(e) => {
          log::error!("invalid authorization header: {}", e);
          return Err(
            InternalError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR),
          );
        }
      };
      let claims_value = match jwt_api_client(&state.config, authorization_string)
        .jwt_is_valid(&state.config.object_storage.tenant_client_id.to_string())
        .await
      {
        Ok(claims_value) => claims_value,
        Err(e) => {
          log::error!("failed to validate authorization header: {:?}", e);
          return Err(
            InternalError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR),
          );
        }
      };
      let claims = match serde_json::from_value(serde_json::Value::Object(
        serde_json::Map::from_iter(claims_value.into_iter()),
      )) {
        Ok(claims) => claims,
        Err(e) => {
          log::error!("failed to parse auth jwt claims response: {:?}", e);
          return Err(
            InternalError::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR),
          );
        }
      };
      return Ok(Self { claims });
    }
    Err(InternalError::unauthorized().with_error(AUTHORIZATION_HEADER, REQUIRED_ERROR))
  }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Claims {
  pub r#type: String,
  pub exp: i64,
  pub iat: i64,
  pub nbf: i64,
  pub iss: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub aud: Option<String>,
  pub sub_type: String,
  pub sub: i64,
  pub app: i64,
  pub scopes: Vec<String>,
}
