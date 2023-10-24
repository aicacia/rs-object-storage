use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct FileRow {
  pub id: i32,
  pub key: String,
  pub hash: String,
  pub size: i32,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct File {
  pub id: i32,
  pub key: String,
  pub hash: String,
  pub size: i32,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<FileRow> for File {
  fn from(file: FileRow) -> Self {
    Self {
      id: file.id,
      key: file.key,
      hash: file.hash,
      size: file.size,
      created_at: file.created_at,
      updated_at: file.updated_at,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, IntoParams, Validate)]
pub struct FilesAndFoldersQuery {
  pub key: String,
}

#[derive(Serialize, Deserialize, Clone, IntoParams, Validate)]
pub struct FileQuery {
  pub key: String,
}

#[derive(Serialize, Deserialize, Clone, ToSchema, Validate)]
pub struct SignedTokenRequest {
  pub expires: DateTime<Utc>,
  pub key: String,
}

#[derive(MultipartForm, ToSchema)]
#[multipart(deny_unknown_fields, duplicate_field = "deny")]
pub struct FileUploadRequest {
  #[schema(value_type = String)]
  pub key: Text<String>,
  #[schema(value_type = String, format = Binary)]
  pub file: TempFile,
}
