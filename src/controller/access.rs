use actix_web::{
  patch, post,
  web::Path,
  web::{scope, Data, ServiceConfig},
  HttpResponse, Responder,
};
use actix_web_validator::Json;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
  core::{config::get_config, jwt::Claims},
  middleware::auth::Authorization,
  model::{
    access::{AccessRequest, AccessRow, CreateAccessRequest},
    error::Errors,
  },
  service::access::{create_access, reset_access, validate_access},
};

#[utoipa::path(
  context_path = "/access",
  request_body = AccessRequest,
  responses(
    (status = 200, description = "Created a new access id/secret", body = AccessWithExposedSecret),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  )
)]
#[post("/token")]
pub async fn create_token(pool: Data<Pool<Postgres>>, body: Json<AccessRequest>) -> impl Responder {
  let access = match validate_access(pool.as_ref(), &body.id, &body.secret).await {
    Ok(Some(a)) => a,
    Ok(None) => return HttpResponse::Unauthorized().json(Errors::unauthorized()),
    Err(_) => return HttpResponse::InternalServerError().json(Errors::internal_error()),
  };
  let config = get_config();
  let jwt: String = match Claims::new(
    access.id,
    chrono::Utc::now().timestamp() as usize,
    config.jwt.expires_in_seconds,
    &config.server.uri,
  )
  .encode(&config.jwt.secret)
  {
    Ok(jwt) => jwt,
    Err(e) => {
      log::error!("{}", e);
      return HttpResponse::InternalServerError().json(Errors::internal_error());
    }
  };
  HttpResponse::Ok().content_type("text/plain").body(jwt)
}

#[utoipa::path(
  context_path = "/access",
  request_body = CreateAccessRequest,
  responses(
    (status = 200, description = "Created a new access id/secret", body = AccessWithExposedSecret),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 403, description = "Forbidden", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
#[post("")]
pub async fn create(
  pool: Data<Pool<Postgres>>,
  body: Json<CreateAccessRequest>,
  access: AccessRow,
) -> impl Responder {
  if !access.admin {
    return HttpResponse::Forbidden().json(Errors::forbidden());
  }
  let access_with_exposed_secret =
    match create_access(pool.as_ref(), body.admin.unwrap_or(false)).await {
      Ok(a) => a,
      Err(_) => return HttpResponse::InternalServerError().json(Errors::internal_error()),
    };
  HttpResponse::Created().json(access_with_exposed_secret)
}

#[utoipa::path(
  context_path = "/access",
  responses(
    (status = 200, description = "Resets access secret", body = AccessWithExposedSecret),
    (status = 401, description = "Unauthorized", body = Errors),
    (status = 403, description = "Forbidden", body = Errors),
    (status = 500, description = "Internal Server Error", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
#[patch("/{id}")]
pub async fn reset(
  pool: Data<Pool<Postgres>>,
  access: AccessRow,
  path: Path<Uuid>,
) -> impl Responder {
  if !access.admin {
    return HttpResponse::Forbidden().json(Errors::forbidden());
  }
  let id = path.into_inner();
  let access_with_exposed_secret = match reset_access(pool.as_ref(), &id).await {
    Ok(a) => a,
    Err(_) => return HttpResponse::InternalServerError().json(Errors::internal_error()),
  };
  HttpResponse::Ok().json(access_with_exposed_secret)
}

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
  |config: &mut ServiceConfig| {
    config.service(
      scope("/access")
        .service(create_token)
        .service(scope("").wrap(Authorization).service(create).service(reset)),
    );
  }
}
