use axum::{
    routing::{get, post},
    Router,
};

use crate::api::handlers;
use crate::api::AppState;

/// Create the API router with all routes
pub fn api_router() -> Router<AppState> {
    // Blockchain state endpoints
    let blockchain_routes = Router::new()
        .route("/summary", get(handlers::get_blockchain_state_summary))
        .route("/refresh", post(handlers::refresh_blockchain_state));
    
    // Request endpoints
    let request_routes = Router::new()
        .route("/:request_id", get(handlers::get_request_by_id))
        .route("/wallet/:wallet_address", get(handlers::get_requests_by_wallet))
        .route("/deposits", get(handlers::get_deposit_requests))
        .route("/withdrawals", get(handlers::get_withdrawal_requests))
        .route("/borrows", get(handlers::get_borrow_requests));
    
    // User endpoints
    let user_routes = Router::new()
        .route("/:wallet_address", get(handlers::get_user_by_wallet));
    
    // Epoch endpoints
    let epoch_routes = Router::new()
        .route("/:epoch_id", get(handlers::get_epoch_by_id))
        .route("/current", get(handlers::get_current_epoch));
    
    // Combine all routes
    Router::new()
        .nest("/blockchain", blockchain_routes)
        .nest("/requests", request_routes)
        .nest("/users", user_routes)
        .nest("/epochs", epoch_routes)
} 