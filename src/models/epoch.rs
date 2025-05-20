use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Epoch status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum EpochStatus {
    Active,
    Processing,
    Completed,
}

impl fmt::Display for EpochStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpochStatus::Active => write!(f, "active"),
            EpochStatus::Processing => write!(f, "processing"),
            EpochStatus::Completed => write!(f, "completed"),
        }
    }
}

impl Default for EpochStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Epoch model - tracks epoch lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epoch {
    pub id: i32,
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: Option<DateTime<Utc>>,
    pub status: EpochStatus,
    pub processed_at: Option<DateTime<Utc>>,
    pub processing_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Epoch summary model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSummary {
    pub id: i32,
    pub status: EpochStatus,
    pub start_timestamp: DateTime<Utc>,
    pub end_timestamp: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
}

/// Update epoch status request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEpochStatusRequest {
    pub status: EpochStatus,
    pub end_timestamp: Option<DateTime<Utc>>,
    pub processing_tx_hash: Option<String>,
}

/// Process epoch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEpochResult {
    pub epoch_id: i32,
    pub status: EpochStatus,
    pub processed_at: DateTime<Utc>,
    pub deposits_processed: i32,
    pub withdrawals_processed: i32,
    pub borrows_processed: i32,
} 