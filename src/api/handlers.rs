use axum::{
    extract::{Path, State},
    Json,
};

use crate::api::blockchain::{BlockchainStateManager, BlockchainStateSummary, OnChainRequest, OnChainUser, OnChainEpoch};
use crate::api::error::ApiResult;
use crate::api::AppState;
use crate::models::blockchain_request::RequestType;

/// Get blockchain state summary
pub async fn get_blockchain_state_summary(
    State(state): State<AppState>,
) -> ApiResult<Json<BlockchainStateSummary>> {
    let blockchain_state = state.blockchain_state.read().await;
    
    let summary = BlockchainStateSummary {
        current_epoch_id: blockchain_state.current_epoch_id,
        active_requests_count: blockchain_state.requests.values().filter(|r| !r.is_processed).count(),
        processed_requests_count: blockchain_state.requests.values().filter(|r| r.is_processed).count(),
        registered_users_count: blockchain_state.users.len(),
        last_updated: blockchain_state.last_updated,
    };
    
    Ok(Json(summary))
}

/// Get request by ID
pub async fn get_request_by_id(
    State(state): State<AppState>,
    Path(request_id): Path<u128>,
) -> ApiResult<Json<OnChainRequest>> {
    let blockchain_manager = BlockchainStateManager::new(state.blockchain_state);
    let request = blockchain_manager.get_request(request_id).await?;
    
    Ok(Json(request))
}

/// Get requests by wallet address
pub async fn get_requests_by_wallet(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> ApiResult<Json<Vec<OnChainRequest>>> {
    let blockchain_manager = BlockchainStateManager::new(state.blockchain_state);
    let requests = blockchain_manager.get_requests_by_wallet(&wallet_address).await?;
    
    Ok(Json(requests))
}

/// Get user by wallet address
pub async fn get_user_by_wallet(
    State(state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> ApiResult<Json<OnChainUser>> {
    let blockchain_manager = BlockchainStateManager::new(state.blockchain_state);
    let user = blockchain_manager.get_user(&wallet_address).await?;
    
    Ok(Json(user))
}

/// Get epoch by ID
pub async fn get_epoch_by_id(
    State(state): State<AppState>,
    Path(epoch_id): Path<u128>,
) -> ApiResult<Json<OnChainEpoch>> {
    let blockchain_manager = BlockchainStateManager::new(state.blockchain_state);
    let epoch = blockchain_manager.get_epoch(epoch_id).await?;
    
    Ok(Json(epoch))
}

/// Get current epoch
pub async fn get_current_epoch(
    State(state): State<AppState>,
) -> ApiResult<Json<OnChainEpoch>> {
    let blockchain_manager = BlockchainStateManager::new(state.blockchain_state);
    let epoch = blockchain_manager.get_current_epoch().await?;
    
    Ok(Json(epoch))
}

/// Get deposit requests
pub async fn get_deposit_requests(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<OnChainRequest>>> {
    let blockchain_manager = BlockchainStateManager::new(state.blockchain_state);
    let requests = blockchain_manager.get_requests_by_type(RequestType::Deposit).await?;
    
    Ok(Json(requests))
}

/// Get withdrawal requests
pub async fn get_withdrawal_requests(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<OnChainRequest>>> {
    let blockchain_manager = BlockchainStateManager::new(state.blockchain_state);
    let requests = blockchain_manager.get_requests_by_type(RequestType::Withdrawal).await?;
    
    Ok(Json(requests))
}

/// Get borrow requests
pub async fn get_borrow_requests(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<OnChainRequest>>> {
    let blockchain_manager = BlockchainStateManager::new(state.blockchain_state);
    let requests = blockchain_manager.get_requests_by_type(RequestType::Borrow).await?;
    
    Ok(Json(requests))
}

/// Refresh blockchain state
pub async fn refresh_blockchain_state(
    State(state): State<AppState>,
) -> ApiResult<Json<BlockchainStateSummary>> {
    let blockchain_manager = BlockchainStateManager::new(state.blockchain_state.clone());
    
    // Refresh the state
    blockchain_manager.refresh_state().await?;
    
    // Return the updated summary
    get_blockchain_state_summary(State(state)).await
} 