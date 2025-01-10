pub mod object;
pub mod openapi;
pub mod util;

use axum::Router;
use object::OBJECT_TAG;
use openapi::OPENAPI_TAG;
use sqlx::AnyPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use util::UTIL_TAG;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::core::openapi::{SecurityAddon, ServersAddon};

#[derive(Clone)]
pub struct RouterState {
  pub pool: AnyPool,
}

unsafe impl Send for RouterState {}
unsafe impl Sync for RouterState {}

#[derive(OpenApi)]
#[openapi(
  info(license(name = "MIT OR Apache-2.0", identifier = "https://spdx.org/licenses/MIT.html")),
  tags(
    (name = OBJECT_TAG, description = "Object endpoints"),
    (name = UTIL_TAG, description = "Utility endpoints"),
    (name = OPENAPI_TAG, description = "OpenApi endpoints"),
  ),
  modifiers(&SecurityAddon, &ServersAddon)
)]
pub struct ApiDoc;

pub fn create_router(state: RouterState) -> Router {
  let open_api_router = OpenApiRouter::with_openapi(ApiDoc::openapi())
    .merge(object::create_router(state.clone()))
    .merge(util::create_router(state.clone()));

  let openapi = open_api_router.get_openapi().clone();
  open_api_router
    .merge(openapi::create_router(openapi))
    .layer(CorsLayer::very_permissive())
    .layer(TraceLayer::new_for_http())
    .into()
}
