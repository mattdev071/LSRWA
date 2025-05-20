use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

/// User balance model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub active_balance: String,
    pub pending_deposits: String,
    pub pending_withdrawals: String,
    pub total_deposited: String,
    pub total_withdrawn: String,
    pub total_rewards: String,
    pub last_reward_claim_timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Update user balance request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserBalanceRequest {
    pub active_balance: Option<String>,
    pub pending_deposits: Option<String>,
    pub pending_withdrawals: Option<String>,
    pub total_deposited: Option<String>,
    pub total_withdrawn: Option<String>,
    pub total_rewards: Option<String>,
    pub last_reward_claim_timestamp: Option<DateTime<Utc>>,
} 