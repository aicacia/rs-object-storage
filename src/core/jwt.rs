use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::encryption::random_password;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
  pub exp: usize,
  pub iat: usize,
  pub iss: String,
  pub nonce: String,
  pub sub: Uuid,
}

impl Claims {
  pub fn new(sub: Uuid, now_in_seconds: usize, expires_in_seconds: usize, iss: &str) -> Self {
    Self {
      exp: now_in_seconds + expires_in_seconds,
      iat: now_in_seconds,
      iss: iss.to_owned(),
      nonce: random_password(32),
      sub,
    }
  }

  pub fn parse(jwt: &str, secret: &str) -> Result<TokenData<Self>> {
    let token_data = decode::<Self>(
      jwt,
      &DecodingKey::from_secret(secret.as_bytes()),
      &Validation::default(),
    )?;
    Ok(token_data)
  }

  pub fn encode(&self, secret: &str) -> Result<String> {
    let jwt = encode(
      &Header::default(),
      self,
      &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(jwt)
  }
}
