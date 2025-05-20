use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::models::blockchain_request::RequestType;
use crate::api::error::{ApiError, ApiResult};

/// Represents the current state of the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainState {
    /// Current epoch ID
    pub current_epoch_id: u128,
    
    /// Mapping of request ID to on-chain request
    pub requests: HashMap<u128, OnChainRequest>,
    
    /// Mapping of wallet address to user details
    pub users: HashMap<String, OnChainUser>,
    
    /// Mapping of epoch ID to epoch details
    pub epochs: HashMap<u128, OnChainEpoch>,
    
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl Default for BlockchainState {
    fn default() -> Self {
        Self {
            current_epoch_id: 1,
            requests: HashMap::new(),
            users: HashMap::new(),
            epochs: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Represents an on-chain request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainRequest {
    /// On-chain unique identifier
    pub id: u128,
    
    /// Type of request
    pub request_type: RequestType,
    
    /// User's wallet address
    pub wallet_address: String,
    
    /// Request amount
    pub amount: String,
    
    /// Collateral amount (for borrow requests)
    pub collateral_amount: Option<String>,
    
    /// Submission timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Whether the request has been processed
    pub is_processed: bool,
    
    /// Block number when the request was submitted
    pub block_number: u64,
    
    /// Transaction hash of the request
    pub transaction_hash: String,
}

/// Represents an on-chain user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainUser {
    /// User's wallet address
    pub wallet_address: String,
    
    /// Whether the user is registered
    pub is_registered: bool,
    
    /// Whether the user's KYC is approved
    pub is_kyc_approved: bool,
    
    /// User's active balance
    pub active_balance: String,
    
    /// User's pending deposits
    pub pending_deposits: String,
    
    /// User's pending withdrawals
    pub pending_withdrawals: String,
    
    /// User's total rewards
    pub total_rewards: String,
}

/// Represents an on-chain epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainEpoch {
    /// Epoch ID
    pub id: u128,
    
    /// Start timestamp
    pub start_timestamp: DateTime<Utc>,
    
    /// End timestamp (if epoch is closed)
    pub end_timestamp: Option<DateTime<Utc>>,
    
    /// Whether the epoch is active
    pub is_active: bool,
}

/// Interface for blockchain state operations
pub struct BlockchainStateManager {
    state: Arc<RwLock<BlockchainState>>,
}

impl BlockchainStateManager {
    /// Create a new blockchain state manager
    pub fn new(state: Arc<RwLock<BlockchainState>>) -> Self {
        Self { state }
    }
    
    /// Get the current blockchain state
    pub async fn get_state(&self) -> BlockchainState {
        self.state.read().await.clone()
    }
    
    /// Get request by ID
    pub async fn get_request(&self, request_id: u128) -> ApiResult<OnChainRequest> {
        let state = self.state.read().await;
        
        state.requests.get(&request_id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Request with ID {} not found", request_id)))
    }
    
    /// Get all requests for a wallet address
    pub async fn get_requests_by_wallet(&self, wallet_address: &str) -> ApiResult<Vec<OnChainRequest>> {
        let state = self.state.read().await;
        
        let wallet_requests = state.requests.values()
            .filter(|r| r.wallet_address == wallet_address)
            .cloned()
            .collect::<Vec<_>>();
            
        Ok(wallet_requests)
    }
    
    /// Get user by wallet address
    pub async fn get_user(&self, wallet_address: &str) -> ApiResult<OnChainUser> {
        let state = self.state.read().await;
        
        state.users.get(wallet_address)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("User with wallet address {} not found", wallet_address)))
    }
    
    /// Get epoch by ID
    pub async fn get_epoch(&self, epoch_id: u128) -> ApiResult<OnChainEpoch> {
        let state = self.state.read().await;
        
        state.epochs.get(&epoch_id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Epoch with ID {} not found", epoch_id)))
    }
    
    /// Get current epoch
    pub async fn get_current_epoch(&self) -> ApiResult<OnChainEpoch> {
        let state = self.state.read().await;
        
        self.get_epoch(state.current_epoch_id).await
    }
    
    /// Get requests of a specific type
    pub async fn get_requests_by_type(&self, request_type: RequestType) -> ApiResult<Vec<OnChainRequest>> {
        let state = self.state.read().await;
        
        let filtered_requests = state.requests.values()
            .filter(|r| r.request_type == request_type)
            .cloned()
            .collect::<Vec<_>>();
            
        Ok(filtered_requests)
    }
    
    /// Refresh the blockchain state (would be implemented to communicate with the smart contract)
    pub async fn refresh_state(&self) -> ApiResult<()> {
        // This would be implemented to communicate with the smart contract
        // For now, just update the last_updated timestamp
        let mut state = self.state.write().await;
        state.last_updated = Utc::now();
        
        Ok(())
    }
}

/// Response containing the current blockchain state summary
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainStateSummary {
    /// Current epoch ID
    pub current_epoch_id: u128,
    
    /// Count of active requests
    pub active_requests_count: usize,
    
    /// Count of processed requests
    pub processed_requests_count: usize,
    
    /// Count of registered users
    pub registered_users_count: usize,
    
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
} 