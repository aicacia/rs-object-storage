use crate::{core::error::Errors, middleware::authorization::Authorization, model::p2p::P2P};

use axum::response::IntoResponse;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::RouterState;

pub const P2P_TAG: &str = "p2p";

#[utoipa::path(
  get,
  path = "/p2p",
  tags = [P2P_TAG],
  responses(
    (status = 200, description = "P2P response", body = P2P),
    (status = 401, content_type = "application/json", body = Errors),
    (status = 404, content_type = "application/json", body = Errors),
    (status = 500, content_type = "application/json", body = Errors),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn p2p(Authorization { .. }: Authorization) -> impl IntoResponse {
  axum::Json(P2P::default()).into_response()
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new().routes(routes!(p2p)).with_state(state)
}
