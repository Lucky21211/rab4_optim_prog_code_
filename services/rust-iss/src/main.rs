mod clients;
mod config;
mod domain;
mod handlers;
mod repo;
mod routes;
mod services;
use crate::{clients::HttpClient, config::Config, services::AppState};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = Arc::new(Config::from_env()?);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let state = AppState {
        pool,
        http: HttpClient::new(&config)?,
        config,
        jobs: Arc::new(tokio::sync::Semaphore::new(4)),
    };
    services::scheduler::start(state.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!(event = "server_started", port = 3000);
    axum::serve(listener, routes::router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
