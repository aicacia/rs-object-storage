use std::{collections::HashMap, sync::Arc};

use async_tungstenite::tokio::connect_async;
use auth_client::apis::jwt_api;
use dashmap::DashMap;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use webrtc::{
  api::{
    interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
  },
  ice_transport::ice_server::RTCIceServer,
  interceptor::registry::Registry,
  peer_connection::configuration::RTCConfiguration,
};
use webrtc_http::server::RTCListener;
use webrtc_p2p::{peer::SignalMessage, Peer, PeerOptions};

use crate::core::{config::Config, error::InternalError};

use super::auth::{auth_token_configuration, get_service_account_token};

pub async fn serve_peer(
  router: axum::Router,
  config: Arc<Config>,
  cancellation_token: CancellationToken,
) -> Result<(), InternalError> {
  let mut m = MediaEngine::default();
  let registry = register_default_interceptors(Registry::new(), &mut m)?;

  let api = Arc::new(
    APIBuilder::new()
      .with_media_engine(m)
      .with_interceptor_registry(registry)
      .build(),
  );
  let peer_options = PeerOptions {
    connection_config: Some(RTCConfiguration {
      ice_servers: vec![RTCIceServer {
        ..Default::default()
      }],
      ..Default::default()
    }),
    ..Default::default()
  };

  loop {
    tokio::select! {
      _ = cancellation_token.cancelled() => {
        break;
      }
      result = ws_serve_peer(config.as_ref(), api.clone(), peer_options.clone(), router.clone(), cancellation_token.clone()) => match result {
        Ok(_) => {
          break;
        }
        Err(e) => {
          log::error!("failed to serve peer: {}", e);
          tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
      }
    }
  }
  Ok(())
}

async fn ws_serve_peer(
  config: &Config,
  api: Arc<webrtc::api::API>,
  peer_options: PeerOptions,
  router: axum::Router,
  cancellation_token: CancellationToken,
) -> Result<(), InternalError> {
  let ws_server_token = create_p2p_ws_server_token(config).await?;
  let ws_url = format!(
    "{}/server/websocket?token={}",
    config.p2p.ws_uri,
    urlencoding::encode(&ws_server_token)
  );
  let (ws, _) = connect_async(ws_url).await?;
  let socket = Arc::new(Mutex::new(ws));

  let peers: Arc<DashMap<String, Peer>> = Arc::new(DashMap::<String, Peer>::new());
  let serve_cancellation_tokens = Arc::new(DashMap::<String, CancellationToken>::new());

  while let Some(msg_result) = socket.lock().await.next().await {
    let msg = msg_result?;
    if msg.is_close() {
      return Err(InternalError::internal_error().with_application_error("socket closed"));
    }
    let data = msg.into_data().to_vec();
    let json = serde_json::from_slice::<IncomingMessage>(&data)?;

    match json {
      IncomingMessage::Join { from } => {
        let mut peer_options = peer_options.clone();
        peer_options.id = Some(from.clone());
        let peer = Peer::new(api.clone(), peer_options);

        let on_signal_socket = socket.clone();
        let on_signal_from = from.clone();
        peer.on_signal(Box::new(move |data| {
          let msg_json = serde_json::to_string(&OutgoingMessage {
            to: on_signal_from.clone(),
            payload: data,
          })
          .expect("failed to serialize signal message");
          let msg = async_tungstenite::tungstenite::Message::text(msg_json);
          let pinned_socket = on_signal_socket.clone();
          Box::pin(async move {
            pinned_socket
              .lock()
              .await
              .send(msg)
              .await
              .expect("failed to write to websocket");
          })
        }));

        let on_connect_peer = peer.clone();
        let on_connect_router = router.clone();
        let on_connect_cancellation_token = cancellation_token.clone();
        let on_connect_from = from.clone();
        let on_connect_serve_cancellation_tokens = serve_cancellation_tokens.clone();

        peer.on_connect(Box::new(move || {
          let data_channel = on_connect_peer
            .get_data_channel()
            .expect("failed to get data channel");
          let listener = RTCListener::new(data_channel);
          let router = on_connect_router.clone();

          let serve_cancellation_token = CancellationToken::new();
          on_connect_serve_cancellation_tokens
            .insert(on_connect_from.clone(), serve_cancellation_token.clone());

          let cancellation_token = on_connect_cancellation_token.clone();
          let serve_shutdown_signal = async move {
            tokio::select! {
              _ = cancellation_token.cancelled() => {}
              _ = serve_cancellation_token.cancelled() => {}
            };
          };

          let _ = tokio::spawn(async move {
            axum::serve(listener, router.clone())
              .with_graceful_shutdown(serve_shutdown_signal)
              .await
          });

          Box::pin(async {})
        }));

        peers.insert(from, peer);
      }
      IncomingMessage::Leave { from } => {
        if let Some((_, cancellation_token)) = serve_cancellation_tokens.remove(&from) {
          cancellation_token.cancel();
        }
        if let Some((_, peer)) = peers.remove(&from) {
          peer.close().await?;
        }
      }
      IncomingMessage::Message { from, payload } => {
        if let Some(peer) = peers.get(&from) {
          peer.signal(payload).await?;
        }
      }
    }
  }
  Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingMessage {
  #[serde(rename = "join")]
  Join { from: String },
  #[serde(rename = "leave")]
  Leave { from: String },
  #[serde(rename = "message")]
  Message {
    from: String,
    payload: SignalMessage,
  },
}

#[derive(Serialize)]
pub struct OutgoingMessage {
  to: String,
  payload: SignalMessage,
}

#[derive(Serialize)]
pub struct ServerAuthenticateRequest {
  id: String,
  password: String,
}

lazy_static! {
  static ref AUTH_P2P_TOKEN: RwLock<Option<(String, i64)>> = RwLock::new(None);
}

async fn create_p2p_ws_server_token(config: &Config) -> Result<String, InternalError> {
  let body = ServerAuthenticateRequest {
    id: config.p2p.id.clone(),
    password: config.p2p.password.clone(),
  };
  let p2p_ws_server_token = reqwest::Client::new()
    .post(format!("{}/server", config.p2p.api_uri))
    .bearer_auth(create_p2p_token(config).await?)
    .json(&body)
    .send()
    .await?
    .text()
    .await?;
  Ok(p2p_ws_server_token)
}

fn create_p2p_claims(config: &Config) -> (HashMap<String, serde_json::Value>, i64) {
  let now = chrono::Utc::now().timestamp();
  let expires_seconds = 5 * 60;
  let expires_at = now + expires_seconds;
  let mut claims = HashMap::new();
  claims.insert("iss".to_owned(), serde_json::json!("P2P"));
  claims.insert("iat".to_owned(), serde_json::json!(now));
  claims.insert("exp".to_owned(), serde_json::json!(expires_at));
  claims.insert("aud".to_owned(), serde_json::json!("P2P"));
  claims.insert("sub".to_owned(), serde_json::json!(config.p2p.id));
  (claims, expires_at)
}

async fn create_p2p_token(config: &Config) -> Result<String, InternalError> {
  let now = chrono::Utc::now().timestamp();
  if let Some((token, expires_at)) = AUTH_P2P_TOKEN.read().await.as_ref() {
    if now < *expires_at {
      return Ok(token.clone());
    }
  }
  let mut auth_p2p_token = AUTH_P2P_TOKEN.write().await;

  let (claims, expires_at) = create_p2p_claims(config);
  let token = create_jwt(config, &config.p2p.tenant_client_id, claims).await?;

  auth_p2p_token.replace((token.clone(), expires_at));

  Ok(token)
}

async fn create_jwt(
  config: &Config,
  tenant_client_id: &uuid::Uuid,
  claims: HashMap<String, serde_json::Value>,
) -> Result<String, InternalError> {
  let service_account_token = match get_service_account_token(config, tenant_client_id).await {
    Ok(token) => token,
    Err(e) => {
      log::info!("Error getting service account token: {:?}", e);
      return Err(InternalError::internal_error());
    }
  };
  let configuration = auth_token_configuration(config, &service_account_token);
  let jwt = match jwt_api::create_jwt(&configuration, claims).await {
    Ok(jwt) => jwt,
    Err(e) => {
      log::info!("Error creating JWT: {:?}", e);
      return Err(InternalError::internal_error());
    }
  };
  Ok(jwt.access_token)
}
