use crate::model::error::Errors;
use crate::{
  core::openapi::SecurityAddon, model::access as access_model, model::error as error_model,
  model::files as files_model, model::util as util_model,
};
use actix_cors::Cors;
use actix_web::{
  body::MessageBody,
  dev::{ServiceFactory, ServiceRequest, ServiceResponse},
  error,
  middleware::Logger,
  web, App,
};
use actix_web_validator::JsonConfig;

use sqlx::{Pool, Postgres};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::controller::{access, files, util};

#[derive(OpenApi)]
#[openapi(
  paths(
    util::health,
    util::version,
    files::index,
    files::show,
    files::create,
    files::edit,
    files::contents,
    access::create_token,
    access::create,
    access::reset,
  ),
  components(
    schemas(
      util_model::Version,
      util_model::Health,
      error_model::Errors,
      error_model::Message,
      error_model::Messages,
      files_model::File,
      files_model::FileUploadRequest,
      access_model::AccessRequest,
      access_model::CreateAccessRequest,
      access_model::AccessWithExposedSecret,
    )
  ),
  tags(
    (name = "util", description = "Utility endpoints"),
    (name = "files", description = "Files/Folders endpoints"),
    (name = "access", description = "Access endpoints"),
  ),
  modifiers(&SecurityAddon)
)]
struct ApiDoc;

pub fn create_app(
  pool: &Pool<Postgres>,
) -> App<
  impl ServiceFactory<
    ServiceRequest,
    Response = ServiceResponse<impl MessageBody>,
    Config = (),
    InitError = (),
    Error = error::Error,
  >,
> {
  let openapi = ApiDoc::openapi();

  let json_config =
    JsonConfig::default().error_handler(|err, _req| Errors::from_validation_error(err).into());

  App::new()
    .app_data(json_config)
    .app_data(web::Data::new(pool.clone()))
    .wrap(Logger::default())
    .wrap(
      Cors::default()
        .allow_any_header()
        .allow_any_method()
        .allow_any_origin()
        .expose_any_header()
        .supports_credentials(),
    )
    .configure(util::configure())
    .configure(files::configure())
    .configure(access::configure())
    .service(
      SwaggerUi::new("/api-docs/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
    )
}
