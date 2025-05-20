pub mod routes;
pub mod handlers;
pub mod blockchain;
pub mod error;
pub mod kyc;

use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::DbPools;

/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    /// Database connection pools
    pub db: DbPools,
    
    /// Blockchain state
    pub blockchain_state: Arc<RwLock<blockchain::BlockchainState>>,
    
    /// KYC service factory
    pub kyc_factory: Arc<kyc::KycServiceFactory>,
}

/// Create the application router
pub fn create_router(state: AppState) -> Router {
    routes::api_router().with_state(state)
} 