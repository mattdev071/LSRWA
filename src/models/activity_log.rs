use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Uuid;

/// Activity log model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub activity_type: String,
    pub description: Option<String>,
    pub data: Option<Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Create activity log request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActivityLogRequest {
    pub user_id: Option<Uuid>,
    pub activity_type: String,
    pub description: Option<String>,
    pub data: Option<Value>,
    pub ip_address: Option<String>,
}

/// Activity log filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogFilter {
    pub user_id: Option<Uuid>,
    pub activity_type: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
} 