use std::{net::SocketAddr, str::FromStr};

use axum::Router;
use clap::Parser;
use object_storage::{
  core::{
    config::{get_config, init_config},
    database::init_pool,
    error::Errors,
  },
  router::{create_router, RouterState},
  service::peer::serve_peer,
};
use sqlx::Executor;
use tokio::fs::create_dir_all;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {}

#[tokio::main]
async fn main() -> Result<(), Errors> {
  dotenv::dotenv().ok();
  sqlx::any::install_default_drivers();

  let config = init_config().await?;

  create_dir_all(&config.objects_dir).await?;

  let level = tracing::Level::from_str(&config.log_level).unwrap_or(tracing::Level::DEBUG);
  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
          "{}={level},tower_http={level},axum::rejection=trace",
          env!("CARGO_PKG_NAME"),
          level = level.as_str().to_lowercase()
        )
        .into()
      }),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

  let pool = init_pool().await?;

  let _args = Args::parse();

  let cancellation_token = CancellationToken::new();

  let router = create_router(RouterState { pool: pool.clone() });
  let serve_handle = tokio::spawn(serve(router.clone(), cancellation_token.clone()));
  let serve_peer_handle = if config.p2p.enabled {
    Some(tokio::spawn(serve_peer(router, cancellation_token.clone())))
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
  cleanup_pool(pool).await;

  Ok(())
}

async fn cleanup_pool(pool: sqlx::AnyPool) {
  match pool.acquire().await {
    Ok(conn) => match conn.backend_name().to_lowercase().as_str() {
      "sqlite" => {
        log::info!("Optimizing database");
        match pool
          .execute("PRAGMA analysis_limit=400; PRAGMA optimize;")
          .await
        {
          Ok(_) => {
            log::info!("Optimized database");
          }
          Err(e) => {
            log::error!("Error optimizing database: {}", e);
          }
        }
      }
      _ => {}
    },
    Err(e) => {
      log::error!("Error acquiring connection: {}", e);
    }
  }
  pool.close().await;
}

async fn serve(router: Router, cancellation_token: CancellationToken) -> Result<(), Errors> {
  let serve_shutdown_signal = async move {
    cancellation_token.cancelled().await;
  };
  let config = get_config();

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
      .map_err(|e| Errors::internal_error().with_application_error(e.to_string()))
  };

  #[cfg(unix)]
  let terminate = async {
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
      Ok(mut signal) => match signal.recv().await {
        Some(_) => Ok(()),
        None => Ok(()),
      },
      Err(e) => Err(Errors::internal_error().with_application_error(e.to_string())),
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
