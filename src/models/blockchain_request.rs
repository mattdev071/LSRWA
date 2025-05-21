use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

/// Request types enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum RequestType {
    Deposit,
    Withdrawal,
    Borrow,
}

impl ToString for RequestType {
    fn to_string(&self) -> String {
        match self {
            RequestType::Deposit => "deposit".to_string(),
            RequestType::Withdrawal => "withdrawal".to_string(),
            RequestType::Borrow => "borrow".to_string(),
        }
    }
}

/// Blockchain request model - mirrors on-chain request data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainRequest {
    pub id: i32,
    pub request_type: RequestType,
    pub on_chain_id: i64,
    pub wallet_address: String,
    pub user_id: Option<Uuid>,
    pub amount: String,
    pub collateral_amount: Option<String>,
    pub submission_timestamp: DateTime<Utc>,
    pub is_processed: bool,
    pub block_number: i64,
    pub transaction_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Batch processing event model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestProcessingEvent {
    pub id: i32,
    pub epoch_id: i32,
    pub processing_type: RequestType,
    pub processed_count: i32,
    pub transaction_hash: String,
    pub block_number: i64,
    pub processing_timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request execution event model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestExecutionEvent {
    pub id: i32,
    pub request_id: i64,
    pub wallet_address: String,
    pub amount: String,
    pub transaction_hash: String,
    pub block_number: i64,
    pub execution_timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Batch processing item status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum BatchItemStatus {
    Included,
    Processed,
    Failed,
}

/// Batch processing item model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingItem {
    pub id: i32,
    pub processing_event_id: i32,
    pub request_id: i64,
    pub request_type: RequestType,
    pub status: BatchItemStatus,
    pub created_at: DateTime<Utc>,
}

/// Create blockchain request data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordBlockchainRequestDto {
    pub request_type: RequestType,
    pub on_chain_id: i64,
    pub wallet_address: String,
    pub amount: String,
    pub collateral_amount: Option<String>,
    pub block_number: i64,
    pub transaction_hash: String,
}

/// Create batch processing event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordBatchProcessingDto {
    pub epoch_id: i32,
    pub processing_type: RequestType,
    pub request_ids: Vec<i64>,
    pub transaction_hash: String,
    pub block_number: i64,
}

/// Create execution event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordExecutionEventDto {
    pub request_id: i64,
    pub wallet_address: String,
    pub amount: String,
    pub transaction_hash: String,
    pub block_number: i64,
}

/// New blockchain request - used for creating a new request
#[derive(Debug, Clone)]
pub struct NewBlockchainRequest {
    pub request_type: RequestType,
    pub on_chain_id: i64,
    pub wallet_address: String,
    pub amount: f64,
    pub collateral_amount: Option<f64>,
    pub timestamp: chrono::NaiveDateTime,
    pub is_processed: bool,
    pub block_number: i64,
    pub transaction_hash: String,
} 