use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;

use crate::{
  core::{
    error::{Errors, INVALID_ERROR, REQUIRED_ERROR},
    openapi::AUTHORIZATION_HEADER,
  },
  router::RouterState,
  service::auth::{auth_is_jwt_valid, Claims},
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
  type Rejection = Errors;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    if let Some(authorization_header_value) = parts.headers.get(AUTHORIZATION_HEADER) {
      let authorization_string = match authorization_header_value.to_str() {
        Ok(authorization_string) => {
          if authorization_string.len() < TOKEN_TYPE_BEARER.len() + 1 {
            return Err(Errors::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
          }
          &authorization_string[(TOKEN_TYPE_BEARER.len() + 1)..]
        }
        Err(e) => {
          log::error!("invalid authorization header: {}", e);
          return Err(Errors::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
        }
      };
      let claims = match auth_is_jwt_valid(authorization_string).await {
        Ok(claims) => claims,
        Err(e) => {
          log::error!("failed to validate authorization header: {}", e);
          return Err(Errors::unauthorized().with_error(AUTHORIZATION_HEADER, INVALID_ERROR));
        }
      };
      return Ok(Self { claims });
    }
    Err(Errors::unauthorized().with_error(AUTHORIZATION_HEADER, REQUIRED_ERROR))
  }
}
