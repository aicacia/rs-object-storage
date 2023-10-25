use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use super::encryption::random_password;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessClaims {
  pub exp: i64,
  pub iat: i64,
  pub iss: String,
  pub nonce: String,
  pub access_id: Uuid,
}

impl AccessClaims {
  pub fn new(access_id: Uuid, now_in_seconds: i64, expires_in_seconds: i64, iss: &str) -> Self {
    Self {
      exp: now_in_seconds + expires_in_seconds,
      iat: now_in_seconds,
      iss: iss.to_owned(),
      nonce: random_password(32),
      access_id,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignedTokenClaims {
  pub exp: i64,
  pub iat: i64,
  pub iss: String,
  pub nonce: String,
  pub file_id: i32,
}

impl SignedTokenClaims {
  pub fn new(file_id: i32, now_in_seconds: i64, expires_in_seconds: i64, iss: &str) -> Self {
    Self {
      exp: now_in_seconds + expires_in_seconds,
      iat: now_in_seconds,
      iss: iss.to_owned(),
      nonce: random_password(32),
      file_id,
    }
  }
}

pub fn parse_jwt<T>(jwt: &str, secret: &str) -> Result<TokenData<T>>
where
  T: DeserializeOwned,
{
  let token_data = decode::<T>(
    jwt,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(),
  )?;
  Ok(token_data)
}

pub fn encode_jwt<T>(claims: &T, secret: &str) -> Result<String>
where
  T: Serialize,
{
  let jwt = encode(
    &Header::default(),
    claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )?;
  Ok(jwt)
}
