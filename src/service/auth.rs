use std::{collections::HashMap, sync::Arc};

use auth_client::{
  apis::{
    configuration::{ApiKey, Configuration},
    JwtApi, JwtApiClient, TokenApi, TokenApiClient,
  },
  models::{
    token_request_service_account::GrantType, JwtRequest, Token, TokenRequest,
    TokenRequestServiceAccount,
  },
};
use hyper_util::{
  client::legacy::{connect::HttpConnector, Client},
  rt::TokioExecutor,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::core::{config::get_config, error::InternalError};

lazy_static! {
  static ref SERVICE_ACCOUNT_TOKEN: RwLock<Option<(Token, i64)>> = RwLock::new(None);
  static ref CLIENT: Client<HttpConnector, String> =
    Client::builder(TokioExecutor::new()).build_http();
}

pub fn jwt_api_client(token: &str) -> impl JwtApi {
  let mut configuration = Configuration::with_client(CLIENT.clone());
  configuration.api_key = Some(ApiKey {
    prefix: Some("Bearer".to_owned()),
    key: token.to_owned(),
  });
  JwtApiClient::new(Arc::new(configuration))
}

pub fn token_api_client() -> impl TokenApi {
  TokenApiClient::new(Arc::new(Configuration::with_client(CLIENT.clone())))
}

pub async fn create_jwt(
  claims: HashMap<String, serde_json::Value>,
) -> Result<String, InternalError> {
  let service_account_token = get_service_account_token().await?;
  let jwt_api = jwt_api_client(&service_account_token);
  let jwt = jwt_api
    .create_jwt(JwtRequest {
      tenant_id: get_config().p2p.tenant_id,
      claims,
    })
    .await?;
  Ok(jwt)
}

pub async fn get_service_account_token() -> Result<String, auth_client::apis::Error> {
  let now = chrono::Utc::now().timestamp();
  if let Some((token, iss_at)) = SERVICE_ACCOUNT_TOKEN.read().await.as_ref() {
    if now < iss_at + token.expires_in {
      return Ok(token.access_token.clone());
    }
  }
  let mut service_account_token = SERVICE_ACCOUNT_TOKEN.write().await;

  let token = create_service_account_token().await?;
  let access_token = token.access_token.clone();

  service_account_token.replace((token, now));

  Ok(access_token)
}

async fn create_service_account_token() -> Result<Token, auth_client::apis::Error> {
  let config = get_config();
  let token_request = TokenRequest::TokenRequestServiceAccount(TokenRequestServiceAccount {
    grant_type: GrantType::ServiceAccount,
    client_id: config.auth.service_account.client_id,
    client_secret: config.auth.service_account.client_secret,
  });
  token_api_client().token(token_request).await
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Claims {
  #[serde(rename = "type")]
  pub kind: String,
  pub exp: i64,
  pub iat: i64,
  pub nbf: i64,
  pub iss: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub aud: Option<String>,
  #[serde(rename = "sub_type")]
  pub sub_kind: String,
  pub sub: i64,
  pub app: i64,
  pub scopes: Vec<String>,
}
