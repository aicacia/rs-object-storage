use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::core::config::get_config;

lazy_static! {
  static ref SERVICE_ACCOUNT_TOKEN: RwLock<Option<(Token, i64)>> = RwLock::new(None);
}

#[derive(Serialize)]
pub struct JWTRequest {
  pub tenant_id: i64,
  pub claims: serde_json::Map<String, serde_json::Value>,
}

pub async fn create_jwt(
  claims: serde_json::Map<String, serde_json::Value>,
) -> Result<String, reqwest::Error> {
  let config = get_config();
  let body = JWTRequest {
    tenant_id: config.p2p.tenant_id,
    claims,
  };
  let service_account_token = get_service_account_token().await?;
  let jwt = reqwest::Client::new()
    .post(format!("{}/jwt", config.auth.uri))
    .bearer_auth(service_account_token)
    .json(&body)
    .send()
    .await?
    .text()
    .await?;

  Ok(jwt)
}

#[derive(Debug, Serialize)]
#[serde(tag = "grant_type")]
pub enum TokenRequest {
  #[serde(rename = "service-account")]
  ServiceAccount {
    client_id: uuid::Uuid,
    client_secret: uuid::Uuid,
  },
}

pub async fn get_service_account_token() -> Result<String, reqwest::Error> {
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

#[derive(Debug, Deserialize)]
pub struct Token {
  pub access_token: String,
  pub token_type: String,
  pub issued_token_type: Option<String>,
  pub expires_in: i64,
  pub scope: Option<String>,
  pub refresh_token: Option<String>,
  pub refresh_token_expires_in: Option<i64>,
  pub id_token: Option<String>,
}

async fn create_service_account_token() -> Result<Token, reqwest::Error> {
  let config = get_config();
  let body = TokenRequest::ServiceAccount {
    client_id: config.auth.service_account.client_id,
    client_secret: config.auth.service_account.client_secret,
  };
  reqwest::Client::new()
    .post(format!("{}/token", config.auth.uri))
    .header("Tenant-ID", config.auth.tenant_client_id.to_string())
    .json(&body)
    .send()
    .await?
    .json::<Token>()
    .await
}

#[derive(Deserialize)]
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

pub async fn auth_is_jwt_valid(token: &str) -> Result<Claims, reqwest::Error> {
  let config = get_config();
  reqwest::Client::new()
    .get(format!("{}/jwt", config.auth.uri))
    .bearer_auth(token)
    .header("Tenant-ID", config.auth.tenant_client_id.to_string())
    .send()
    .await?
    .json::<Claims>()
    .await
}
