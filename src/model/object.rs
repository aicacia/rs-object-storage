use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::repository::object::ObjectRow;

use super::util::Pagination;

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
  pub r#type: Option<String>,
}

#[derive(ToSchema)]
pub struct UploadPartRequest {
  #[schema(format = Binary, content_media_type = "application/octet-stream")]
  pub part: String,
}

#[derive(Serialize, ToSchema)]
pub struct UploadResponse {
  pub written: usize,
}

#[derive(Deserialize, ToSchema)]
pub struct MoveObjectRequest {
  pub path: String,
  pub r#type: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ObjectInstance {
  pub id: i64,
  pub path: String,
  pub r#type: Option<String>,
  pub size: u64,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<ObjectRow> for ObjectInstance {
  fn from(row: ObjectRow) -> Self {
    Self {
      id: row.id,
      path: row.path,
      r#type: row.r#type,
      size: row.size as u64,
      updated_at: DateTime::<Utc>::from_timestamp(row.updated_at, 0).unwrap_or_default(),
      created_at: DateTime::<Utc>::from_timestamp(row.created_at, 0).unwrap_or_default(),
    }
  }
}

pub type ObjectInstancePagination = Pagination<ObjectInstance>;
