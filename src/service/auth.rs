use auth_client::{
  apis::{
    configuration::{ApiKey, Configuration},
    token_api::{self, TokenError},
    Error,
  },
  models::{
    token_request_service_account::GrantType, Token, TokenRequest, TokenRequestServiceAccount,
  },
};
use dashmap::DashMap;

use crate::core::config::Config;

lazy_static! {
  static ref SERVICE_ACCOUNT_TOKENS: DashMap<uuid::Uuid, (Token, i64)> = DashMap::new();
}

pub fn auth_token_configuration(config: &Config, access_token: &str) -> Configuration {
  let mut configuration = Configuration::default();
  configuration.base_path = config.auth.uri.to_owned();
  configuration.oauth_access_token = Some(access_token.to_owned());
  configuration
}

pub fn auth_tenant_configuration(config: &Config, tenant_client_id: &uuid::Uuid) -> Configuration {
  let mut configuration = Configuration::default();
  configuration.base_path = config.auth.uri.to_owned();
  configuration.api_key = Some(ApiKey {
    prefix: None,
    key: tenant_client_id.to_string(),
  });
  configuration
}

pub async fn get_service_account_token(
  config: &Config,
  tenant_client_id: &uuid::Uuid,
) -> Result<String, Error<TokenError>> {
  let now = chrono::Utc::now().timestamp();
  if let Some(entry) = SERVICE_ACCOUNT_TOKENS.get(tenant_client_id) {
    if now <= entry.1 + entry.0.expires_in - 5 {
      return Ok(entry.0.access_token.clone());
    }
  }
  let token = create_service_account_token(config, tenant_client_id).await?;
  let access_token = token.access_token.clone();

  SERVICE_ACCOUNT_TOKENS.insert(tenant_client_id.clone(), (token, now));

  Ok(access_token)
}

async fn create_service_account_token(
  config: &Config,
  tenant_client_id: &uuid::Uuid,
) -> Result<Token, Error<TokenError>> {
  let token_request = TokenRequest::TokenRequestServiceAccount(TokenRequestServiceAccount {
    grant_type: GrantType::ServiceAccount,
    client_id: config.auth.service_account.client_id,
    client_secret: config.auth.service_account.client_secret,
  });
  let configuration = auth_tenant_configuration(config, tenant_client_id);
  token_api::token(&configuration, token_request).await
}
