pub mod routes;
pub mod handlers;
pub mod blockchain;
pub mod error;

use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::DbPool;

/// Shared application state that will be available to all request handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db: DbPool,
    /// Blockchain state cache
    pub blockchain_state: Arc<RwLock<blockchain::BlockchainState>>,
}

/// Create and configure the API router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", routes::api_router())
        .with_state(state)
} 