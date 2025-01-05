use std::usize;

use crate::{
  core::{
    config::get_config,
    error::{Errors, INTERNAL_ERROR, INVALID_ERROR, NOT_FOUND_ERROR, REQUEST_BODY},
  },
  middleware::{authorization::Authorization, json::Json},
  model::{
    object::{CreateObjectRequest, Object, ObjectQuery, ObjectsQuery, MoveObjectRequest, UploadPartRequest},
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
    get_objects,
    get_object_by_path,
    get_object_by_id,
    read_object_by_id,
    read_object_by_path,
    create_object,
    append_object,
    move_object,
    delete_object
  ),
  tags(
    (name = "object", description = "Objects"),
  )
)]
pub struct ApiDoc;

#[utoipa::path(
  get,
  path = "objects",
  tags = ["object"],
  params(
    OffsetAndLimit,
    ObjectsQuery,
  ),
  responses(
    (status = 200, content_type = "application/json", body = Pagination<Object>),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn get_objects(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Query(offset_and_limit_query): Query<OffsetAndLimit>,
  Query(objects_query): Query<ObjectsQuery>,
) -> impl IntoResponse {
  let objects = match repository::object::get_objects_and_folders(
    &state.pool,
    objects_query.path.as_ref().map(String::as_str),
    offset_and_limit_query.limit,
    offset_and_limit_query.offset,
  )
  .await
  {
    Ok(objects) => objects,
    Err(err) => {
      log::error!("Error getting objects from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  axum::Json(Pagination {
    has_more: objects.len() == offset_and_limit_query.limit.unwrap_or(usize::MAX),
    items: objects.into_iter().map(Object::from).collect(),
  })
  .into_response()
}

#[utoipa::path(
  get,
  path = "objects/by-path",
  tags = ["object"],
  params(
    ObjectQuery,
  ),
  responses(
    (status = 200, content_type = "application/json", body = Object),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn get_object_by_path(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Query(object_query): Query<ObjectQuery>,
) -> impl IntoResponse {
  let object_row =
    match repository::object::get_object_by_path(&state.pool, &object_query.path).await {
      Ok(Some(object_row)) => object_row,
      Ok(None) => {
        log::error!("Object not found: {}", object_query.path);
        return Errors::not_found()
          .with_error("path", NOT_FOUND_ERROR)
          .into_response();
      }
      Err(err) => {
        log::error!("Error getting objects from database: {}", err);
        return Errors::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };

  axum::Json(Object::from(object_row)).into_response()
}

#[utoipa::path(
  get,
  path = "objects/{object_id}",
  tags = ["object"],
  responses(
    (status = 200, content_type = "application/json", body = Object),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn get_object_by_id(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(object_id): Path<i64>,
) -> impl IntoResponse {
  let object_row = match repository::object::get_object_by_id(&state.pool, object_id).await {
    Ok(Some(object_row)) => object_row,
    Ok(None) => {
      log::error!("Object not found: {}", object_id);
      return Errors::not_found()
        .with_error("object_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting objects from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  axum::Json(Object::from(object_row)).into_response()
}

#[utoipa::path(
  get,
  path = "objects/{object_id}/read",
  tags = ["object"],
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
pub async fn read_object_by_id(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(object_id): Path<i64>,
) -> impl IntoResponse {
  let object_row = match repository::object::get_object_by_id(&state.pool, object_id).await {
    Ok(Some(object)) => object,
    Ok(None) => {
      log::error!("Object not found: {}", object_id);
      return Errors::not_found()
        .with_error("object_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting objects from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let config = get_config();
  let objects_path = std::path::Path::new(&config.objects_dir);
  let object_path = objects_path.join(object_row.id.to_string());
  let object = match fs::OpenOptions::new()
    .create(false)
    .read(true)
    .open(object_path)
    .await
  {
    Ok(object) => object,
    Err(err) => {
      log::error!("Error opening object: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };
  let content_type = object_row
    .kind
    .unwrap_or_else(|| "application/octet-stream".to_owned());
  let content_disposition = format!("attachment; objectname={:?}", object_row.path);
  (
    [
      (header::CONTENT_TYPE, content_type),
      (header::CONTENT_DISPOSITION, content_disposition),
    ],
    Body::from_stream(ReaderStream::new(object)),
  )
    .into_response()
}

#[utoipa::path(
  get,
  path = "objects/by-path/read",
  tags = ["object"],
  params(
    ObjectQuery,
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
pub async fn read_object_by_path(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Query(query): Query<ObjectQuery>,
) -> impl IntoResponse {
  let object_row = match repository::object::get_object_by_path(&state.pool, &query.path).await {
    Ok(Some(object)) => object,
    Ok(None) => {
      log::error!("Object not found: {}", query.path);
      return Errors::not_found()
        .with_error("path", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting objects from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let config = get_config();
  let objects_path = std::path::Path::new(&config.objects_dir);
  let object_path = objects_path.join(object_row.id.to_string());
  let object = match fs::OpenOptions::new()
    .create(false)
    .read(true)
    .open(object_path)
    .await
  {
    Ok(object) => object,
    Err(err) => {
      log::error!("Error opening object: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };
  let content_type = object_row
    .kind
    .unwrap_or_else(|| "application/octet-stream".to_owned());
  let content_disposition = format!("attachment; objectname={:?}", object_row.path);
  (
    [
      (header::CONTENT_TYPE, content_type),
      (header::CONTENT_DISPOSITION, content_disposition),
    ],
    Body::from_stream(ReaderStream::new(object)),
  )
    .into_response()
}

#[utoipa::path(
  post,
  path = "objects",
  tags = ["object"],
  request_body = CreateObjectRequest,
  responses(
    (status = 201, content_type = "application/json", body = Object),
    (status = 400, content_type = "application/json", body = Errors),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn create_object(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Json(body): Json<CreateObjectRequest>,
) -> impl IntoResponse {
  let object_row = match service::object::create_object(&state.pool, body.path, body.kind).await {
    Ok(object_row) => object_row,
    Err(err) => {
      log::error!("Error creating object in database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  (StatusCode::CREATED, axum::Json(Object::from(object_row))).into_response()
}

#[utoipa::path(
  put,
  path = "objects/{object_id}/append",
  tags = ["object"],
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
pub async fn append_object(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(object_id): Path<i64>,
  mut multipart: Multipart,
) -> impl IntoResponse {
  let object_row = match repository::object::get_object_by_id(&state.pool, object_id).await {
    Ok(Some(object)) => object,
    Ok(None) => {
      log::error!("Object not found: {}", object_id);
      return Errors::not_found()
        .with_error("object_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting objects from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };
  let config = get_config();
  let objects_path = std::path::Path::new(&config.objects_dir);
  let object_path = objects_path.join(object_row.id.to_string());
  let mut object = match fs::OpenOptions::new()
    .create(false)
    .read(false)
    .append(true)
    .open(object_path)
    .await
  {
    Ok(object) => object,
    Err(err) => {
      log::error!("Error opening object: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let mut written = 0;
  loop {
    match multipart.next_field().await {
      Ok(Some(field)) => match field.bytes().await {
        Ok(bytes) => {
          match service::object::append_object(&state.pool, object_id, &mut object, bytes).await {
            Ok(w) => {
              written += w;
            }
            Err(err) => {
              log::error!("Error appending object: {}", err);
              return Errors::internal_error()
                .with_application_error(INTERNAL_ERROR)
                .into_response();
            }
          }
        }
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
  path = "objects/{object_id}/move",
  tags = ["object"],
  request_body = MoveObjectRequest,
  responses(
    (status = 200, content_type = "application/json", body = Object),
    (status = 400, content_type = "application/json", body = Errors),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn move_object(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(object_id): Path<i64>,
  Json(body): Json<MoveObjectRequest>,
) -> impl IntoResponse {
  let object_row = match repository::object::update_object_path(
    &state.pool,
    object_id,
    body.path,
    body.kind,
  )
  .await
  {
    Ok(Some(object)) => object,
    Ok(None) => {
      log::error!("Object not found: {}", object_id);
      return Errors::not_found()
        .with_error("object_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error getting objects from database: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };
  axum::Json(Object::from(object_row)).into_response()
}

#[utoipa::path(
  delete,
  path = "objects/{object_id}",
  tags = ["object"],
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
pub async fn delete_object(
  State(state): State<RouterState>,
  Authorization { .. }: Authorization,
  Path(object_id): Path<i64>,
) -> impl IntoResponse {
  match service::object::delete_object(&state.pool, object_id).await {
    Ok(Some(_)) => {}
    Ok(None) => {
      log::error!("Object not found: {}", object_id);
      return Errors::not_found()
        .with_error("object_id", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(err) => {
      log::error!("Error deleting object: {}", err);
      return Errors::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  }
  (StatusCode::NO_CONTENT, ()).into_response()
}

pub fn create_router(state: RouterState) -> Router {
  Router::new()
    .route("/objects", get(get_objects))
    .route("/objects/by-path", get(get_object_by_path))
    .route("/objects/{object_id}", get(get_object_by_id))
    .route("/objects/{object_id}/read", get(read_object_by_id))
    .route("/objects/by-path/read", get(read_object_by_path))
    .route("/objects", post(create_object))
    .route("/objects/{object_id}/append", put(append_object))
    .route("/objects/{object_id}/move", put(move_object))
    .route("/objects/{object_id}", delete(delete_object))
    .with_state(state)
}
