use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod blockchain;
pub mod error;
pub mod handlers;
pub mod routes;

use blockchain::BlockchainState;
use crate::db::DbPools;

/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    /// Database connection pools
    pub db: DbPools,
    
    /// Blockchain state
    pub blockchain_state: Arc<RwLock<BlockchainState>>,
}

/// Create the application router
pub fn create_router(state: AppState) -> Router {
    routes::api_router().with_state(state)
} 