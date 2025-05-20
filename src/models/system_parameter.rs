use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

/// System parameter model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemParameter {
    pub id: i32,
    pub parameter_name: String,
    pub parameter_value: String,
    pub description: Option<String>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create system parameter request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSystemParameterRequest {
    pub parameter_name: String,
    pub parameter_value: String,
    pub description: Option<String>,
    pub updated_by: Option<Uuid>,
}

/// Update system parameter request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSystemParameterRequest {
    pub parameter_value: String,
    pub description: Option<String>,
    pub updated_by: Option<Uuid>,
}

/// System parameters cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemParametersCache {
    pub reward_apr_bps: i32,
    pub epoch_duration_seconds: i64,
    pub max_epochs_before_liquidation: i32,
    pub collateral_ratio_bps: i32,
    pub min_deposit_amount: String,
    pub min_withdrawal_amount: String,
    pub min_borrow_amount: String,
}

impl Default for SystemParametersCache {
    fn default() -> Self {
        Self {
            reward_apr_bps: 500,
            epoch_duration_seconds: 604800,
            max_epochs_before_liquidation: 2,
            collateral_ratio_bps: 15000,
            min_deposit_amount: "100000000".to_string(),
            min_withdrawal_amount: "100000000".to_string(),
            min_borrow_amount: "1000000000".to_string(),
        }
    }
} 