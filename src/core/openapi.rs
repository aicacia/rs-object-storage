use std::sync::Arc;

use utoipa::{
  openapi::{
    security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Server,
  },
  Modify,
};

use super::config::Config;

pub const AUTHORIZATION_HEADER: &str = "Authorization";

pub struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    let components = openapi.components.as_mut().unwrap();
    components.add_security_scheme(
      "Authorization",
      SecurityScheme::Http(
        HttpBuilder::new()
          .scheme(HttpAuthScheme::Bearer)
          .bearer_format("JWT")
          .build(),
      ),
    );
  }
}

pub struct ServersAddon {
  config: Arc<Config>,
}

impl ServersAddon {
  pub fn new(config: Arc<Config>) -> Self {
    Self { config }
  }
}

impl Modify for ServersAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    openapi
      .servers
      .get_or_insert(Vec::default())
      .push(Server::new(self.config.server.url.clone()));
  }
}
