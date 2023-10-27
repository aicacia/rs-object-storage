use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Serialize, Deserialize, Clone, ToSchema, Validate)]
pub struct UploadRequest {
  pub expires: Option<DateTime<Utc>>,
  #[validate(length(min = 1, max = 4096))]
  pub key: String,
}

#[derive(MultipartForm, ToSchema)]
#[multipart(deny_unknown_fields, duplicate_field = "deny")]
pub struct UploadPartRequest {
  #[schema(value_type = String, format = Binary)]
  pub file: TempFile,
}
