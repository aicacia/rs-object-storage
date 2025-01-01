pub mod file;
pub mod openapi;
pub mod util;

use axum::Router;
use sqlx::AnyPool;
use tower_http::cors::CorsLayer;
use utoipa::{openapi::Server, OpenApi};

use crate::core::{config::get_config, openapi::SecurityAddon};

#[derive(Clone)]
pub struct RouterState {
  pub pool: AnyPool,
}

unsafe impl Send for RouterState {}
unsafe impl Sync for RouterState {}

#[derive(OpenApi)]
#[openapi(
  info(license(name = "MIT OR Apache-2.0", identifier = "https://spdx.org/licenses/MIT.html")),
  nest(
    (path = "/", api = file::ApiDoc),
    (path = "/", api = openapi::ApiDoc),
    (path = "/", api = util::ApiDoc),
  ),
  modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub fn create_router(state: RouterState) -> Router {
  let config = get_config();

  let mut doc = ApiDoc::openapi();
  doc
    .servers
    .get_or_insert(Vec::default())
    .push(Server::new(config.server.url.clone()));

  Router::new()
    .merge(file::create_router(state.clone()))
    .merge(openapi::create_router(doc))
    .merge(util::create_router(state.clone()))
    .layer(CorsLayer::very_permissive())
    .layer(
      tower_http::trace::TraceLayer::new_for_http().make_span_with(
        |request: &axum::http::Request<_>| {
          tracing::info_span!(
            "http",
            method = ?request.method(),
            path = ?request.uri(),
          )
        },
      ),
    )
}
