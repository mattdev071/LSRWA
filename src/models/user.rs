use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

/// KYC status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum KycStatus {
    Pending,
    Approved,
    Rejected,
}

impl Default for KycStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// User model - stores user information and KYC status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub email: Option<String>,
    pub kyc_status: KycStatus,
    pub kyc_timestamp: Option<DateTime<Utc>>,
    pub kyc_reference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create user request data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub wallet_address: String,
    pub email: Option<String>,
}

/// Update user request data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub kyc_status: Option<KycStatus>,
    pub kyc_timestamp: Option<DateTime<Utc>>,
    pub kyc_reference: Option<String>,
}

/// User data with balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithBalance {
    pub id: Uuid,
    pub wallet_address: String,
    pub email: Option<String>,
    pub kyc_status: KycStatus,
    pub active_balance: String,
    pub pending_deposits: String,
    pub pending_withdrawals: String,
    pub total_rewards: String,
} 