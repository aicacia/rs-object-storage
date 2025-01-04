use atomicoption::AtomicOption;
use config::ConfigError;
use serde::Deserialize;
use std::{
  net::IpAddr,
  sync::{atomic::Ordering, Arc},
};

static CONFIG: AtomicOption<Arc<Config>> = AtomicOption::none();

pub async fn init_config() -> Result<Arc<Config>, ConfigError> {
  let config = Arc::new(Config::new().await?);
  CONFIG.store(Ordering::SeqCst, config.clone());
  Ok(config)
}

pub fn get_config() -> Arc<Config> {
  CONFIG
    .as_ref(Ordering::Relaxed)
    .expect("Config not initialized")
    .clone()
}

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
pub struct P2PConfig {
  pub enabled: bool,
  pub tenant_id: i64,
  pub ws_uri: String,
  pub api_uri: String,
  pub id: String,
  pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthServiceAccountConfig {
  pub client_id: uuid::Uuid,
  pub client_secret: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
  pub uri: String,
  pub tenant_id: i64,
  pub tenant_client_id: uuid::Uuid,
  pub service_account: AuthServiceAccountConfig,
}

#[derive(Debug, Deserialize)]
pub struct Config {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub p2p: P2PConfig,
  pub auth: AuthConfig,
  pub files_dir: String,
  pub log_level: String,
}

impl Config {
  pub async fn new() -> Result<Self, ConfigError> {
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
      // P2P
      .set_default("p2p.enabled", true)?
      .set_default("p2p.ws_uri", "wss://p2p.aicacia.com".to_owned())?
      .set_default("p2p.api_uri", "https://p2p.aicacia.com".to_owned())?
      .set_default("p2p.id", uuid::Uuid::new_v4().to_string())?
      .set_default("p2p.password", uuid::Uuid::new_v4().to_string())?
      // Auth
      .set_default("auth.uri", "https://api.auth.aicacia.com".to_owned())?
      // Defaults
      .set_default("files_dir", "./files")?
      .set_default("log_level", "debug")?
      .add_source(config::File::with_name("./config.json"))
      .add_source(config::Environment::with_prefix("APP"))
      .build()?;

    let config = config_builder.try_deserialize()?;
    Ok(config)
  }
}
