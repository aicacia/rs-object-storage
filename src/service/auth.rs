use std::collections::HashMap;

use auth_client::{
  apis::{
    configuration::{ApiKey, Configuration},
    jwt_api, token_api,
  },
  models::{
    token_request_service_account::GrantType, Token, TokenRequest, TokenRequestServiceAccount,
  },
};
use dashmap::DashMap;

use crate::core::{config::Config, error::InternalError};

lazy_static! {
  static ref SERVICE_ACCOUNT_TOKENS: DashMap<uuid::Uuid, (Token, i64)> = DashMap::new();
}

pub fn create_access_token_configuration(config: &Config, access_token: &str) -> Configuration {
  let mut configuration = Configuration::default();
  configuration.base_path = config.auth.uri.to_owned();
  configuration.oauth_access_token = Some(access_token.to_owned());
  configuration
}

pub fn create_tenant_configuration(
  config: &Config,
  tenant_client_id: &uuid::Uuid,
) -> Configuration {
  let mut configuration = Configuration::default();
  configuration.base_path = config.auth.uri.to_owned();
  configuration.api_key = Some(ApiKey {
    prefix: None,
    key: tenant_client_id.to_string(),
  });
  configuration
}

pub async fn create_jwt(
  config: &Config,
  tenant_client_id: &uuid::Uuid,
  claims: HashMap<String, serde_json::Value>,
) -> Result<String, InternalError> {
  let service_account_token = get_service_account_token(config, tenant_client_id).await?;
  let configuration = create_access_token_configuration(config, &service_account_token);
  let jwt = match jwt_api::create_jwt(&configuration, claims).await {
    Ok(jwt) => jwt,
    Err(e) => {
      log::info!("Error creating JWT: {:?}", e);
      return Err(InternalError::internal_error());
    }
  };
  Ok(jwt.access_token)
}

async fn get_service_account_token(
  config: &Config,
  tenant_client_id: &uuid::Uuid,
) -> Result<String, InternalError> {
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
) -> Result<Token, InternalError> {
  let token_request = TokenRequest::TokenRequestServiceAccount(TokenRequestServiceAccount {
    grant_type: GrantType::ServiceAccount,
    client_id: config.auth.service_account.client_id,
    client_secret: config.auth.service_account.client_secret,
  });
  let configuration = create_tenant_configuration(config, tenant_client_id);
  match token_api::token(&configuration, token_request).await {
    Ok(token) => Ok(token),
    Err(e) => {
      log::info!("Error creating service account token: {:?}", e);
      Err(InternalError::internal_error())
    }
  }
}
