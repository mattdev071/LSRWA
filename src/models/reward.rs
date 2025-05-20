use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::fmt;

/// Reward status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum RewardStatus {
    Pending,
    Claimed,
    Expired,
}

impl fmt::Display for RewardStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RewardStatus::Pending => write!(f, "pending"),
            RewardStatus::Claimed => write!(f, "claimed"),
            RewardStatus::Expired => write!(f, "expired"),
        }
    }
}

impl Default for RewardStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// User reward model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReward {
    pub id: Uuid,
    pub user_id: Uuid,
    pub epoch_id: i32,
    pub amount: String,
    pub apr_bps: i32,
    pub status: RewardStatus,
    pub claim_timestamp: Option<DateTime<Utc>>,
    pub claim_transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create user reward request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRewardRequest {
    pub user_id: Uuid,
    pub epoch_id: i32,
    pub amount: String,
    pub apr_bps: i32,
}

/// Update user reward status request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRewardStatusRequest {
    pub status: RewardStatus,
    pub claim_timestamp: Option<DateTime<Utc>>,
    pub claim_transaction_hash: Option<String>,
}

/// User rewards summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRewardsSummary {
    pub user_id: Uuid,
    pub total_pending: String,
    pub total_claimed: String,
    pub total_lifetime: String,
    pub last_claim_timestamp: Option<DateTime<Utc>>,
} 