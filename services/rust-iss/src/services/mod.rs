use crate::{clients::HttpClient, config::Config};
use sqlx::PgPool;
use std::sync::Arc;
pub mod collection;
pub mod scheduler;
pub mod telemetry;
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub http: HttpClient,
    pub config: Arc<Config>,
    pub jobs: Arc<tokio::sync::Semaphore>,
}
