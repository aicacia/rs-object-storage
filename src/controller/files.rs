use actix_files::NamedFile;
use actix_multipart::form::MultipartForm;
use actix_web::{
  get, post, put,
  web::{self, scope, Data, Query, ServiceConfig},
  HttpRequest, HttpResponse, Responder,
};
use actix_web_validator::Json;
use sqlx::{Pool, Postgres};
use std::path::Path;

use crate::{
  core::{
    config::get_config,
    jwt::{encode_jwt, parse_jwt, SignedTokenClaims},
  },
  middleware::auth::AccessAuthorization,
  model::{
    error::Errors,
    files::{File, FileQuery, FileUploadRequest, FilesAndFoldersQuery, SignedTokenRequest},
  },
  service::files::{
    copy_file_and_hash, create_file, get_file_by_id, get_file_by_key, get_file_key_sha256,
    get_files_and_folders, update_file,
  },
};

#[utoipa::path(
  context_path = "/files",
  params(
    FilesAndFoldersQuery,
  ),
  responses(
    (status = 200, description = "List files and folders", body = Vec<File>),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 403, description = "Forbidden", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  ),
  security(
    ("AccessAuthorization" = [])
  )
)]
#[get("/list")]
pub async fn index(
  pool: Data<Pool<Postgres>>,
  query: Query<FilesAndFoldersQuery>,
) -> impl Responder {
  let files_and_folders = match get_files_and_folders(pool.as_ref(), &query.key).await {
    Ok(f) => f,
    Err(e) => {
      log::error!("Error listing files and folders: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };
  let files_and_folders_response: Vec<File> =
    files_and_folders.into_iter().map(Into::into).collect();
  HttpResponse::Ok().json(files_and_folders_response)
}

#[utoipa::path(
  context_path = "/files",
  params(
    FileQuery,
  ),
  responses(
    (status = 200, description = "Fetched file", body = File),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 403, description = "Forbidden", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors)
  ),
  security(
    ("AccessAuthorization" = [])
  )
)]
#[get("")]
pub async fn show(pool: Data<Pool<Postgres>>, query: Query<FileQuery>) -> impl Responder {
  let file = match get_file_by_key(pool.as_ref(), &query.key).await {
    Ok(Some(f)) => f,
    Ok(None) => return HttpResponse::NotFound().json(Errors::not_found()),
    Err(e) => {
      log::error!("Failed to get file by key: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };
  let file_response: File = file.into();
  HttpResponse::Ok().json(file_response)
}

#[utoipa::path(
  context_path = "/files",
  request_body(content = FileUploadRequest, content_type = "multipart/form-data"),
  responses(
    (status = 201, description = "Created file", body = File),
    (status = 400, description = "File exists", body = Errors),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 403, description = "Forbidden", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  ),
  security(
    ("AccessAuthorization" = [])
  )
)]
#[post("")]
pub async fn create(
  pool: Data<Pool<Postgres>>,
  payload: MultipartForm<FileUploadRequest>,
) -> impl Responder {
  let config = get_config();

  let dest_path =
    Path::new(&config.files.files_folder).join(get_file_key_sha256(payload.key.as_str()));

  if tokio::fs::try_exists(&dest_path).await.unwrap_or(true) {
    log::error!("Trying to overwrite a file");
    return HttpResponse::BadRequest().json(Errors::from("file_exists"));
  }

  let size = payload.file.size;
  let hash = match copy_file_and_hash(payload.file.file.path(), &dest_path).await {
    Ok(h) => h,
    Err(e) => {
      log::error!("Error copying file: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };

  let file = match create_file(pool.as_ref(), payload.key.as_str(), &hash, size as i32).await {
    Ok(f) => f,
    Err(e) => {
      println!("Error creating file: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };

  let file_response: File = file.into();
  HttpResponse::Created().json(file_response)
}

#[utoipa::path(
  context_path = "/files",
  request_body(content = FileUploadRequest, content_type = "multipart/form-data"),
  responses(
    (status = 200, description = "Updated file", body = File),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 403, description = "Forbidden", body = Errors),
    (status = 404, description = "File not found", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  ),
  security(
    ("AccessAuthorization" = [])
  )
)]
#[put("")]
pub async fn edit(
  pool: Data<Pool<Postgres>>,
  payload: MultipartForm<FileUploadRequest>,
) -> impl Responder {
  let config = get_config();

  let file = match get_file_by_key(pool.as_ref(), &payload.key).await {
    Ok(Some(f)) => f,
    Ok(None) => return HttpResponse::NotFound().json(Errors::not_found()),
    Err(e) => {
      log::error!("Failed to get file by key: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };

  let dest_path = Path::new(&config.files.files_folder).join(get_file_key_sha256(&file.key));

  let size = payload.file.size;
  let hash = match copy_file_and_hash(payload.file.file.path(), &dest_path).await {
    Ok(h) => h,
    Err(e) => {
      log::error!("Error copying file: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };

  let file = match update_file(pool.as_ref(), &file.key, &hash, size as i32).await {
    Ok(f) => f,
    Err(e) => {
      println!("Error updating file: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };

  let file_response: File = file.into();
  HttpResponse::Ok().json(file_response)
}

#[utoipa::path(
  context_path = "/files",
  params(
    FileQuery,
  ),
  responses(
    (status = 200, description = "Fetched file contents", body = [u8], content_type = "application/octet-stream"),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 403, description = "Forbidden", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors)
  ),
  security(
    ("AccessAuthorization" = [])
  )
)]
#[get("/contents")]
pub async fn contents(
  req: HttpRequest,
  pool: Data<Pool<Postgres>>,
  query: Query<FileQuery>,
) -> impl Responder {
  let config = get_config();
  let file = match get_file_by_key(pool.as_ref(), &query.key).await {
    Ok(Some(f)) => f,
    Ok(None) => return HttpResponse::NotFound().json(Errors::not_found()),
    Err(e) => {
      log::error!("Failed to get file by key: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };
  let path = Path::new(&config.files.files_folder).join(get_file_key_sha256(&file.key));
  let named_file = match NamedFile::open(path) {
    Ok(f) => f,
    Err(e) => {
      log::error!("Failed to open file: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };
  named_file.into_response(&req)
}

#[utoipa::path(
  context_path = "/files",
  request_body = SignedTokenRequest,
  responses(
    (status = 200, description = "Created a new signed token", content_type="text/plain", body = String),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  ),
  security(
    ("AccessAuthorization" = [])
  )
)]
#[post("/signed-token")]
pub async fn signed_token(
  pool: Data<Pool<Postgres>>,
  body: Json<SignedTokenRequest>,
) -> impl Responder {
  let file = match get_file_by_key(pool.as_ref(), &body.key).await {
    Ok(Some(f)) => f,
    Ok(None) => return HttpResponse::NotFound().json(Errors::not_found()),
    Err(e) => {
      log::error!("Failed to get file by key: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };
  let config = get_config();
  let now = chrono::Utc::now().timestamp();
  let expires_in_seconds = body.expires.timestamp() - now;
  let jwt: String = match encode_jwt(
    &SignedTokenClaims::new(
      file.id,
      chrono::Utc::now().timestamp(),
      expires_in_seconds,
      &config.server.uri,
    ),
    &config.jwt.secret,
  ) {
    Ok(jwt) => jwt,
    Err(e) => {
      log::error!("{}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };
  HttpResponse::Ok().content_type("text/plain").body(jwt)
}

#[utoipa::path(
  context_path = "/files",
  responses(
    (status = 200, description = "Fetched file contents", body = [u8], content_type = "application/octet-stream"),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 403, description = "Forbidden", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors)
  )
)]
#[get("/{signed_token}")]
pub async fn signed_token_contents(
  req: HttpRequest,
  pool: Data<Pool<Postgres>>,
  path: web::Path<String>,
) -> impl Responder {
  let jwt = path.into_inner();
  let token_data = match parse_jwt::<SignedTokenClaims>(&jwt, &get_config().jwt.secret) {
    Ok(c) => c,
    Err(err) => {
      log::error!("Error parsing token: {}", err);
      return HttpResponse::NotFound().json(Errors::unauthorized());
    }
  };

  let config = get_config();
  let file = match get_file_by_id(pool.as_ref(), token_data.claims.file_id).await {
    Ok(Some(f)) => f,
    Ok(None) => return HttpResponse::NotFound().json(Errors::not_found()),
    Err(e) => {
      log::error!("Failed to get file by key: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };
  let path = Path::new(&config.files.files_folder).join(get_file_key_sha256(&file.key));
  let named_file = match NamedFile::open(path) {
    Ok(f) => f,
    Err(e) => {
      log::error!("Failed to open file: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };
  named_file.into_response(&req)
}

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
  |config: &mut ServiceConfig| {
    config.service(
      scope("/files").service(signed_token_contents).service(
        scope("")
          .wrap(AccessAuthorization)
          .service(index)
          .service(show)
          .service(create)
          .service(edit)
          .service(contents)
          .service(signed_token),
      ),
    );
  }
}
