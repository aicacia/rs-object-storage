use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
  pub exp: usize,
  pub iat: usize,
  pub iss: String,
  pub nonce: String,
  pub sub: i32,
}

impl Claims {
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
