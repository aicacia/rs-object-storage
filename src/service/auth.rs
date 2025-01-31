use std::{collections::HashMap, sync::Arc};

use auth_client::{
  apis::{
    configuration::{ApiKey, Configuration},
    JwtApi, JwtApiClient, TokenApi, TokenApiClient,
  },
  models::{
    token_request_service_account::GrantType, Token, TokenRequest, TokenRequestServiceAccount,
  },
};
use dashmap::DashMap;
use hyper_util::{
  client::legacy::{connect::HttpConnector, Client},
  rt::TokioExecutor,
};

use crate::core::{config::Config, error::InternalError};

lazy_static! {
  static ref SERVICE_ACCOUNT_TOKENS: DashMap<uuid::Uuid, (Token, i64)> = DashMap::new();
  static ref CLIENT: Client<HttpConnector, String> =
    Client::builder(TokioExecutor::new()).build_http();
}

pub fn jwt_api_client(config: &Config, access_token: &str) -> impl JwtApi {
  let mut configuration = Configuration::with_client(CLIENT.clone());
  configuration.base_path = config.auth.uri.to_owned();
  configuration.oauth_access_token = Some(access_token.to_owned());
  JwtApiClient::new(Arc::new(configuration))
}

pub fn token_api_client(config: &Config, tenant_client_id: &uuid::Uuid) -> impl TokenApi {
  let mut configuration = Configuration::with_client(CLIENT.clone());
  configuration.base_path = config.auth.uri.to_owned();
  configuration.api_key = Some(ApiKey {
    prefix: None,
    key: tenant_client_id.to_string(),
  });
  TokenApiClient::new(Arc::new(configuration))
}

pub async fn create_jwt(
  config: &Config,
  tenant_client_id: &uuid::Uuid,
  claims: HashMap<String, serde_json::Value>,
) -> Result<String, InternalError> {
  let service_account_token = get_service_account_token(config, tenant_client_id).await?;
  let jwt_api = jwt_api_client(config, &service_account_token);
  let jwt = jwt_api.create_jwt(claims).await?;
  Ok(jwt.access_token)
}

pub async fn get_service_account_token(
  config: &Config,
  tenant_client_id: &uuid::Uuid,
) -> Result<String, auth_client::apis::Error> {
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
) -> Result<Token, auth_client::apis::Error> {
  let token_request = TokenRequest::TokenRequestServiceAccount(TokenRequestServiceAccount {
    grant_type: GrantType::ServiceAccount,
    client_id: config.auth.service_account.client_id,
    client_secret: config.auth.service_account.client_secret,
  });
  token_api_client(config, tenant_client_id)
    .token(token_request)
    .await
}
