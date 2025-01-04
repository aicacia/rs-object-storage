use std::usize;

use crate::{
  core::{
    config::get_config,
    error::{Errors, INTERNAL_ERROR, INVALID_ERROR, NOT_FOUND_ERROR, REQUEST_BODY},
  },
  middleware::{authorization::Authorization, json::Json},
  model::{
    file::{CreateFileRequest, File, FileQuery, FilesQuery, MoveFileRequest, UploadPartRequest},
    util::{OffsetAndLimit, Pagination},
  },
  repository, service,
};

use axum::{
  body::Body,
  extract::{Multipart, Path, Query, State},
  http::{header, StatusCode},
  response::IntoResponse,
  routing::{delete, get, post, put},
  Router,
};
use tokio::fs;
use tokio_util::io::ReaderStream;
use utoipa::OpenApi;

use super::RouterState;

#[derive(OpenApi)]
#[openapi(
  paths(
    get_files,
    get_file_by_path,
    get_file_by_id,
    read_file_by_id,
    read_file_by_path,
    create_file,
    append_file,
    move_file,
    delete_file
  ),
  tags(
    (name = "file", description = "Files"),
  )
)]
pub struct ApiDoc;

#[utoipa::path(
  get,
  path = "files",
  tags = ["file"],
  params(
    OffsetAndLimit,
    FilesQuery,
  ),
  responses(
    (status = 200, content_type = "application/json", body = Pagination<File>),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn get_files(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Query(offset_and_limit_query): Query<OffsetAndLimit>,
  Query(files_query): Query<FilesQuery>,
) -> impl IntoResponse {
  let files = match repository::file::get_files_and_folders(
    &state.pool,
    files_query.path.as_ref().map(String::as_str),
    offset_and_limit_query.limit,
    offset_and_limit_query.offset,
  )
  .await
  {
    Ok(files) => files,
    Err(err) => {
      log::error!("Error getting files from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  axum::Json(Pagination {
    has_more: files.len() == offset_and_limit_query.limit.unwrap_or(usize::MAX),
    items: files.into_iter().map(File::from).collect(),
  })
  .into_response()
}

#[utoipa::path(
  get,
  path = "files/by-path",
  tags = ["file"],
  params(
    FileQuery,
  ),
  responses(
    (status = 200, content_type = "application/json", body = File),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn get_file_by_path(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Query(file_query): Query<FileQuery>,
) -> impl IntoResponse {
  let file_row = match repository::file::get_file_by_path(&state.pool, &file_query.path).await {
    Ok(Some(file_row)) => file_row,
    Ok(None) => {
      log::error!("File not found: {}", file_query.path);
      return Errors::not_found()
        .with_error("path", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting files from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  axum::Json(File::from(file_row)).into_response()
}

#[utoipa::path(
  get,
  path = "files/{file_id}",
  tags = ["file"],
  responses(
    (status = 200, content_type = "application/json", body = File),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn get_file_by_id(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(file_id): Path<i64>,
) -> impl IntoResponse {
  let file_row = match repository::file::get_file_by_id(&state.pool, file_id).await {
    Ok(Some(file_row)) => file_row,
    Ok(None) => {
      log::error!("File not found: {}", file_id);
      return Errors::not_found()
        .with_error("file_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting files from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  axum::Json(File::from(file_row)).into_response()
}

#[utoipa::path(
  get,
  path = "files/{file_id}/read",
  tags = ["file"],
  responses(
    (status = 200, content_type = "*/*"),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn read_file_by_id(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(file_id): Path<i64>,
) -> impl IntoResponse {
  let file_row = match repository::file::get_file_by_id(&state.pool, file_id).await {
    Ok(Some(file)) => file,
    Ok(None) => {
      log::error!("File not found: {}", file_id);
      return Errors::not_found()
        .with_error("file_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting files from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let config = get_config();
  let files_path = std::path::Path::new(&config.files_dir);
  let file_path = files_path.join(file_row.id.to_string());
  let file = match fs::OpenOptions::new()
    .create(false)
    .read(true)
    .open(file_path)
    .await
  {
    Ok(file) => file,
    Err(err) => {
      log::error!("Error opening file: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };
  let content_type = file_row
    .kind
    .unwrap_or_else(|| "application/octet-stream".to_owned());
  let content_disposition = format!("attachment; filename={:?}", file_row.path);
  (
    [
      (header::CONTENT_TYPE, content_type),
      (header::CONTENT_DISPOSITION, content_disposition),
    ],
    Body::from_stream(ReaderStream::new(file)),
  )
    .into_response()
}

#[utoipa::path(
  get,
  path = "files/by-path/read",
  tags = ["file"],
  params(
    FileQuery,
  ),
  responses(
    (status = 200, content_type = "*/*"),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn read_file_by_path(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Query(query): Query<FileQuery>,
) -> impl IntoResponse {
  let file_row = match repository::file::get_file_by_path(&state.pool, &query.path).await {
    Ok(Some(file)) => file,
    Ok(None) => {
      log::error!("File not found: {}", query.path);
      return Errors::not_found()
        .with_error("path", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting files from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let config = get_config();
  let files_path = std::path::Path::new(&config.files_dir);
  let file_path = files_path.join(file_row.id.to_string());
  let file = match fs::OpenOptions::new()
    .create(false)
    .read(true)
    .open(file_path)
    .await
  {
    Ok(file) => file,
    Err(err) => {
      log::error!("Error opening file: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };
  let content_type = file_row
    .kind
    .unwrap_or_else(|| "application/octet-stream".to_owned());
  let content_disposition = format!("attachment; filename={:?}", file_row.path);
  (
    [
      (header::CONTENT_TYPE, content_type),
      (header::CONTENT_DISPOSITION, content_disposition),
    ],
    Body::from_stream(ReaderStream::new(file)),
  )
    .into_response()
}

#[utoipa::path(
  post,
  path = "files",
  tags = ["file"],
  request_body = CreateFileRequest,
  responses(
    (status = 201, content_type = "application/json", body = File),
    (status = 400, content_type = "application/json", body = Errors),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn create_file(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Json(body): Json<CreateFileRequest>,
) -> impl IntoResponse {
  let file_row = match service::file::create_file(&state.pool, body.path, body.kind).await {
    Ok(file_row) => file_row,
    Err(err) => {
      log::error!("Error creating file in database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  (StatusCode::CREATED, axum::Json(File::from(file_row))).into_response()
}

#[utoipa::path(
  put,
  path = "files/{file_id}/append",
  tags = ["file"],
  request_body(content = UploadPartRequest, content_type = "multipart/form-data"),
  responses(
    (status = 200, content_type = "application/json", body = usize),
    (status = 400, content_type = "application/json", body = Errors),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn append_file(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(file_id): Path<i64>,
  mut multipart: Multipart,
) -> impl IntoResponse {
  let file_row = match repository::file::get_file_by_id(&state.pool, file_id).await {
    Ok(Some(file)) => file,
    Ok(None) => {
      log::error!("File not found: {}", file_id);
      return Errors::not_found()
        .with_error("file_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting files from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };
  let config = get_config();
  let files_path = std::path::Path::new(&config.files_dir);
  let file_path = files_path.join(file_row.id.to_string());
  let mut file = match fs::OpenOptions::new()
    .create(false)
    .read(false)
    .append(true)
    .open(file_path)
    .await
  {
    Ok(file) => file,
    Err(err) => {
      log::error!("Error opening file: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let mut written = 0;
  loop {
    match multipart.next_field().await {
      Ok(Some(field)) => match field.bytes().await {
        Ok(bytes) => match service::file::append_file(&state.pool, file_id, &mut file, bytes).await
        {
          Ok(w) => {
            written += w;
          }
          Err(err) => {
            log::error!("Error appending file: {}", err);
            return Errors::internal_error()
              .with_application_error(INTERNAL_ERROR)
              .into_response();
          }
        },
        Err(err) => {
          log::error!("Error reading field: {}", err);
          return Errors::bad_request()
            .with_error(REQUEST_BODY, INVALID_ERROR)
            .into_response();
        }
      },
      Ok(None) => {
        break;
      }
      Err(err) => {
        log::error!("Error getting next field: {}", err);
        return Errors::bad_request()
          .with_error(REQUEST_BODY, INVALID_ERROR)
          .into_response();
      }
    }
  }
  axum::Json(written).into_response()
}

#[utoipa::path(
  put,
  path = "files/{file_id}/move",
  tags = ["file"],
  request_body = MoveFileRequest,
  responses(
    (status = 200, content_type = "application/json", body = File),
    (status = 400, content_type = "application/json", body = Errors),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn move_file(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(file_id): Path<i64>,
  Json(body): Json<MoveFileRequest>,
) -> impl IntoResponse {
  let file_row =
    match repository::file::update_file_path(&state.pool, file_id, body.path, body.kind).await {
      Ok(Some(file)) => file,
      Ok(None) => {
        log::error!("File not found: {}", file_id);
        return Errors::not_found()
          .with_error("file_id", NOT_FOUND_ERROR)
          .into_response();
      }
      Err(err) => {
        log::error!("Error getting files from database: {}", err);
        return Errors::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };
  axum::Json(File::from(file_row)).into_response()
}

#[utoipa::path(
  delete,
  path = "files/{file_id}",
  tags = ["file"],
  responses(
    (status = 204),
    (status = 400, content_type = "application/json", body = Errors),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn delete_file(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(file_id): Path<i64>,
) -> impl IntoResponse {
  match service::file::delete_file(&state.pool, file_id).await {
    Ok(Some(_)) => {}
    Ok(None) => {
      log::error!("File not found: {}", file_id);
      return Errors::not_found()
        .with_error("file_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error deleting file: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  }
  (StatusCode::NO_CONTENT, ()).into_response()
}

pub fn create_router(state: RouterState) -> Router {
  Router::new()
    .route("/files", get(get_files))
    .route("/files/by-path", get(get_file_by_path))
    .route("/files/{file_id}", get(get_file_by_id))
    .route("/files/{file_id}/read", get(read_file_by_id))
    .route("/files/by-path/read", get(read_file_by_path))
    .route("/files", post(create_file))
    .route("/files/{file_id}/append", put(append_file))
    .route("/files/{file_id}/move", put(move_file))
    .route("/files/{file_id}", delete(delete_file))
    .with_state(state)
}
