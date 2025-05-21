//! Event types for the indexer service

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::blockchain_request::RequestType;

/// Status of event processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingStatus {
    /// Event is pending processing
    Pending,
    /// Event is being processed
    Processing,
    /// Event has been processed successfully
    Processed,
    /// Event processing failed
    Failed,
    /// Event processing is on hold
    OnHold,
}

/// Type of blockchain event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Deposit request event
    DepositRequest,
    /// Withdrawal request event
    WithdrawalRequest,
    /// Borrow request event
    BorrowRequest,
    /// Request execution event
    RequestExecution,
    /// Batch processing event
    BatchProcessing,
    /// User registration event
    UserRegistration,
    /// Epoch creation event
    EpochCreation,
    /// Epoch closing event
    EpochClosing,
    /// Validation failure event
    ValidationFailure,
}

/// Indexed blockchain event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Type of event
    pub event_type: EventType,
    /// Block number where the event was emitted
    pub block_number: u64,
    /// Transaction hash of the event
    pub transaction_hash: String,
    /// Related request ID (if applicable)
    pub request_id: Option<u128>,
    /// Related wallet address
    pub wallet_address: Option<String>,
    /// Event amount (if applicable)
    pub amount: Option<String>,
    /// Request type (if applicable)
    pub request_type: Option<RequestType>,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Raw event data
    pub raw_data: String,
    /// Processing status
    pub status: ProcessingStatus,
    /// Number of processing attempts
    pub attempts: u32,
    /// Last processing attempt timestamp
    pub last_attempt: Option<DateTime<Utc>>,
    /// Error message from last processing attempt
    pub error_message: Option<String>,
} 