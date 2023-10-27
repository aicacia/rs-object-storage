use std::path::Path;

use actix_multipart::form::MultipartForm;
use actix_web::{
  post, put,
  web::{self, scope, ServiceConfig},
  HttpResponse, Responder,
};
use actix_web_validator::Json;

use crate::{
  core::{
    config::get_config,
    jwt::{encode_jwt, parse_jwt, UploadClaims},
  },
  middleware::auth::AccessAuthorization,
  model::{
    error::Errors,
    uploads::{UploadPartRequest, UploadRequest},
  },
  service::files::copy_file_and_hash,
};

#[utoipa::path(
  context_path = "/uploads",
  request_body = UploadRequest,
  responses(
    (status = 201, description = "Creates a upload token", content_type="text/plain", body = String),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  ),
  security(
    ("AccessAuthorization" = [])
  )
)]
#[post("")]
pub async fn create(body: Json<UploadRequest>) -> impl Responder {
  let config = get_config();
  let now_in_seconds = chrono::Utc::now().timestamp();
  let expires_in_seconds = body
    .expires
    .map(|d| d.timestamp() - now_in_seconds)
    .unwrap_or(config.jwt.expires_in_seconds);
  let claims = UploadClaims::new(
    body.key.clone(),
    now_in_seconds,
    expires_in_seconds,
    config.server.uri.clone(),
  );
  let jwt: String = match encode_jwt(&claims, &config.jwt.secret) {
    Ok(jwt) => jwt,
    Err(e) => {
      log::error!("{}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };

  let upload_folder = Path::new(&config.files.uploads_folder).join(claims.sha256());
  log::info!("Creating upload folder {}", upload_folder.display());
  match tokio::fs::create_dir_all(upload_folder).await {
    Ok(_) => {}
    Err(e) => {
      log::error!("{}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  }

  HttpResponse::Ok().content_type("text/plain").body(jwt)
}

#[utoipa::path(
  context_path = "/uploads",
  request_body(content = UploadPartRequest, content_type = "multipart/form-data"),
  responses(
    (status = 201, description = "Uploaded file part sha256", content_type="text/plain", body = String),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  )
)]
#[put("/{jwt}/{part}")]
pub async fn upload_part(
  path: web::Path<(String, usize)>,
  payload: MultipartForm<UploadPartRequest>,
) -> impl Responder {
  let config = get_config();
  let (jwt, part) = path.into_inner();
  let token_data = match parse_jwt::<UploadClaims>(&jwt, &config.jwt.secret) {
    Ok(c) => c,
    Err(err) => {
      log::error!("Error parsing token: {}", err);
      return HttpResponse::Unauthorized().json(Errors::unauthorized());
    }
  };
  let upload_part_path = Path::new(&config.files.uploads_folder)
    .join(token_data.claims.sha256())
    .join(part.to_string());

  let hash = match copy_file_and_hash(payload.file.file.path(), &upload_part_path).await {
    Ok(h) => h,
    Err(e) => {
      log::error!("Error copying file: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };

  HttpResponse::Ok().content_type("text/plain").body(hash)
}

#[utoipa::path(
  context_path = "/uploads",
  responses(
    (status = 201, description = "Finish upload", body = File),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  )
)]
#[post("/{jwt}/finish")]
pub async fn finish(path: web::Path<String>) -> impl Responder {
  let config = get_config();
  let jwt = path.into_inner();
  let token_data = match parse_jwt::<UploadClaims>(&jwt, &config.jwt.secret) {
    Ok(c) => c,
    Err(err) => {
      log::error!("Error parsing token: {}", err);
      return HttpResponse::Unauthorized().json(Errors::unauthorized());
    }
  };
  let dest_path = Path::new(&config.files.files_folder).join(&token_data.claims.key);
  let upload_parts_path = Path::new(&config.files.uploads_folder).join(token_data.claims.sha256());

  let mut parts: Vec<usize> = vec![];
  match tokio::fs::read_dir(&upload_parts_path).await {
    Ok(mut dir) => {
      while let Ok(Some(item)) = dir.next_entry().await {
        if let Some(Ok(idx)) = item.file_name().to_str().map(str::parse::<usize>) {
          parts.push(idx);
        }
      }
    }
    Err(e) => {
      log::error!("Error parsing token: {}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  }
  parts.sort();

  for idx in parts {
    let upload_part_path = upload_parts_path.join(idx.to_string());
    log::info!("Appending {:?} to {:?}", upload_part_path, dest_path);
  }

  HttpResponse::NoContent().finish()
}

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
  |config: &mut ServiceConfig| {
    config.service(
      scope("/uploads")
        .service(upload_part)
        .service(finish)
        .service(scope("").wrap(AccessAuthorization).service(create)),
    );
  }
}
