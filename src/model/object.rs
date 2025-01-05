use axum::body::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::repository::object::ObjectRow;

#[derive(Deserialize, IntoParams)]
pub struct ObjectsQuery {
  pub path: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct ObjectQuery {
  pub path: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateObjectRequest {
  pub path: String,
  #[serde(rename = "type")]
  pub kind: Option<String>,
}

#[derive(ToSchema)]
#[allow(unused)]
pub struct UploadPartRequest {
  #[schema(value_type = String, format = Binary, content_media_type = "application/octet-stream")]
  pub part: Bytes,
}

#[derive(Deserialize, ToSchema)]
pub struct MoveObjectRequest {
  pub path: String,
  #[serde(rename = "type")]
  pub kind: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Object {
  pub id: i64,
  pub path: String,
  #[serde(rename = "type")]
  pub kind: Option<String>,
  pub size: u64,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<ObjectRow> for Object {
  fn from(row: ObjectRow) -> Self {
    Self {
      id: row.id,
      path: row.path,
      kind: row.kind,
      size: row.size as u64,
      updated_at: DateTime::<Utc>::from_timestamp(row.updated_at, 0).unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(row.created_at, 0).unwrap_or_default(),
    }
  }
}
