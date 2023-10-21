use actix_web::{dev::Payload, FromRequest, HttpMessage, HttpRequest};
use chrono::{DateTime, Utc};
use futures::future::{err, ok};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::error::Errors;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct AccessRow {
  pub id: Uuid,
  pub encrypted_secret: String,
  pub admin: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct Access {
  pub id: Uuid,
  pub admin: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<AccessRow> for Access {
  fn from(access: AccessRow) -> Self {
    Self {
      id: access.id,
      admin: access.admin,
      created_at: access.created_at,
      updated_at: access.updated_at,
    }
  }
}

impl FromRequest for AccessRow {
  type Error = actix_web::Error;
  type Future = futures::future::Ready<Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
    match req.extensions().get::<AccessRow>() {
      Some(access) => ok(access.clone()),
      None => {
        let mut error = Errors::new();
        error.error("access", "invalid");
        err(actix_web::error::ErrorUnauthorized(error))
      }
    }
  }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct AccessWithExposedSecret {
  pub id: Uuid,
  pub secret: String,
  pub admin: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Validate, ToSchema)]
pub struct AccessRequest {
  pub id: Uuid,
  pub secret: String,
}

#[derive(Serialize, Deserialize, Clone, Validate, ToSchema)]
pub struct CreateAccessRequest {
  pub admin: Option<bool>,
}
