use config::ConfigError;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
  pub address: IpAddr,
  pub port: u16,
  pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
  pub url: String,
  pub min_connections: u32,
  pub max_connections: u32,
  pub connect_timeout: u64,
  pub acquire_timeout: u64,
  pub idle_timeout: u64,
  pub max_lifetime: u64,
}

#[derive(Debug, Deserialize)]
pub struct AuthServiceAccountConfig {
  pub client_id: uuid::Uuid,
  pub client_secret: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
  pub uri: String,
  pub service_account: AuthServiceAccountConfig,
}

#[derive(Debug, Deserialize)]
pub struct ObjectStorageConfig {
  pub tenant_client_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct Config {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  #[serde(rename = "object-storage")]
  pub object_storage: ObjectStorageConfig,
  pub auth: AuthConfig,
  pub objects_dir: String,
  pub log_level: String,
}

impl Config {
  pub async fn new(config_path: &str) -> Result<Self, ConfigError> {
    let config_builder = config::Config::builder()
      // Server Defaults
      .set_default("server.address", "0.0.0.0")?
      .set_default("server.port", 3000)?
      .set_default("server.url", "http://localhost:3000")?
      // Database Defaults
      .set_default(
        "database.url",
        std::env::var("DATABASE_URL").unwrap_or_default(),
      )?
      .set_default("database.min_connections", 1)?
      .set_default("database.max_connections", 100)?
      .set_default("database.connect_timeout", 3)?
      .set_default("database.acquire_timeout", 3)?
      .set_default("database.idle_timeout", 5)?
      .set_default("database.max_lifetime", 300)?
      // Auth
      .set_default("auth.uri", "https://api.auth.aicacia.com".to_owned())?
      // Defaults
      .set_default("objects_dir", "./objects")?
      .set_default("log_level", "debug")?
      .add_source(config::File::with_name(config_path))
      .add_source(config::Environment::with_prefix("APP"))
      .build()?;

    let config = config_builder.try_deserialize()?;
    Ok(config)
  }
}
