use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::Router;
use clap::Parser;
use object_storage::{
  core::{
    config::Config,
    database::{close_pool, init_pool},
    error::InternalError,
  },
  router::{create_router, RouterState},
  service::peer::serve_peer,
};
use tokio::fs::create_dir_all;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
  #[arg(short, long, default_value = "./config.json")]
  config: String,
}

#[tokio::main]
async fn main() -> Result<(), InternalError> {
  dotenv::dotenv().ok();
  sqlx::any::install_default_drivers();

  let args = Args::parse();

  let config = Arc::new(Config::new(&args.config).await?);

  create_dir_all(&config.objects_dir).await?;

  let level = tracing::Level::from_str(&config.log_level).unwrap_or(tracing::Level::DEBUG);
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
          "{}={level},tower_http={level},axum::rejection=trace",
          env!("CARGO_PKG_NAME").replace("-", "_"),
          level = level.as_str().to_lowercase()
        )
        .into()
      }),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  let pool = init_pool(config.as_ref()).await?;

  let cancellation_token = CancellationToken::new();

  let router = create_router(RouterState {
    config: config.clone(),
    pool: pool.clone(),
  });
  let serve_handle = tokio::spawn(serve(
    router.clone(),
    config.clone(),
    cancellation_token.clone(),
  ));
  let serve_peer_handle = if config.p2p.enabled {
    Some(tokio::spawn(serve_peer(
      router,
      config.clone(),
      cancellation_token.clone(),
    )))
  } else {
    None
  };

  shutdown_signal(cancellation_token).await;

  match serve_handle.await {
    Ok(_) => {}
    Err(e) => {
      log::error!("Error serving: {}", e);
    }
  }
  if let Some(handle) = serve_peer_handle {
    match handle.await {
      Ok(_) => {}
      Err(e) => {
        log::error!("Error serving peer: {}", e);
      }
    }
  }
  match close_pool().await {
    Ok(_) => {}
    Err(e) => {
      log::error!("Error closing pool: {}", e);
    }
  }

  Ok(())
}

async fn serve(
  router: Router,
  config: Arc<Config>,
  cancellation_token: CancellationToken,
) -> Result<(), InternalError> {
  let serve_shutdown_signal = async move {
    cancellation_token.cancelled().await;
  };

  let listener = tokio::net::TcpListener::bind(&SocketAddr::from((
    config.server.address,
    config.server.port,
  )))
  .await?;
  let local_addr = listener.local_addr()?;
  log::info!("Listening on {}", local_addr);
  axum::serve(
    listener,
    router.into_make_service_with_connect_info::<SocketAddr>(),
  )
  .with_graceful_shutdown(serve_shutdown_signal)
  .await?;
  Ok(())
}

async fn shutdown_signal(cancellation_token: CancellationToken) {
  let ctrl_c = async {
    tokio::signal::ctrl_c()
      .await
      .map_err(|e| InternalError::internal_error().with_application_error(e.to_string()))
  };

  #[cfg(unix)]
  let terminate = async {
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
      Ok(mut signal) => match signal.recv().await {
        Some(_) => Ok(()),
        None => Ok(()),
      },
      Err(e) => Err(InternalError::internal_error().with_application_error(e.to_string())),
    }
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  let result = tokio::select! {
    result = ctrl_c => result,
    result = terminate => result,
  };

  match result {
    Ok(_) => log::info!("Shutdown signal received, shutting down"),
    Err(e) => log::error!("Error receiving shutdown signal: {}", e),
  }

  cancellation_token.cancel();
}
