use axum::body::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::repository::file::FileRow;

#[derive(Deserialize, IntoParams)]
pub struct FilesQuery {
  pub path: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct FileQuery {
  pub path: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateFileRequest {
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
pub struct MoveFileRequest {
  pub path: String,
  #[serde(rename = "type")]
  pub kind: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct File {
  pub id: i64,
  pub path: String,
  #[serde(rename = "type")]
  pub kind: Option<String>,
  pub size: u64,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl From<FileRow> for File {
  fn from(row: FileRow) -> Self {
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
