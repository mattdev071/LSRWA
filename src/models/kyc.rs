use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use crate::models::user::KycStatus;

/// KYC provider enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum KycProvider {
    Internal,
    Sumsub,
    Onfido,
    Shufti,
    Persona,
}

impl Default for KycProvider {
    fn default() -> Self {
        Self::Internal
    }
}

/// KYC verification level enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum KycLevel {
    Basic,
    Advanced,
    Full,
}

impl Default for KycLevel {
    fn default() -> Self {
        Self::Basic
    }
}

/// KYC verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycVerificationRequest {
    pub user_id: Uuid,
    pub wallet_address: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub date_of_birth: Option<String>,
    pub country: Option<String>,
    pub document_type: Option<String>,
    pub document_id: Option<String>,
    pub kyc_level: KycLevel,
    pub provider: KycProvider,
    pub redirect_url: Option<String>,
}

/// KYC verification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycVerificationResponse {
    pub verification_id: String,
    pub user_id: Uuid,
    pub status: KycStatus,
    pub provider: KycProvider,
    pub verification_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// KYC verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycVerificationResult {
    pub verification_id: String,
    pub user_id: Uuid,
    pub status: KycStatus,
    pub provider: KycProvider,
    pub provider_reference: String,
    pub verification_data: serde_json::Value,
    pub verified_at: DateTime<Utc>,
}

/// KYC webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycWebhookPayload {
    pub verification_id: String,
    pub provider_reference: String,
    pub status: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

/// KYC service trait - defines the interface for KYC providers
#[async_trait::async_trait]
pub trait KycService: Send + Sync {
    /// Initialize a new KYC verification process
    async fn initiate_verification(&self, request: KycVerificationRequest) -> anyhow::Result<KycVerificationResponse>;
    
    /// Check the status of a verification
    async fn check_verification_status(&self, verification_id: &str) -> anyhow::Result<KycStatus>;
    
    /// Process webhook data from the KYC provider
    async fn process_webhook(&self, payload: KycWebhookPayload) -> anyhow::Result<KycVerificationResult>;
    
    /// Get verification details
    async fn get_verification_details(&self, verification_id: &str) -> anyhow::Result<KycVerificationResult>;
}

/// Mock KYC service implementation for development and testing
pub struct MockKycService;

#[async_trait::async_trait]
impl KycService for MockKycService {
    async fn initiate_verification(&self, request: KycVerificationRequest) -> anyhow::Result<KycVerificationResponse> {
        // Generate a mock verification ID
        let verification_id = format!("mock-verification-{}", uuid::Uuid::new_v4());
        
        // Create a mock verification URL (would be the real provider's URL in production)
        let verification_url = Some(format!("https://mock-kyc-provider.com/verify/{}", verification_id));
        
        // Return a mock response
        Ok(KycVerificationResponse {
            verification_id,
            user_id: request.user_id,
            status: KycStatus::Pending,
            provider: request.provider,
            verification_url,
            expires_at: Some(Utc::now() + chrono::Duration::days(7)),
        })
    }
    
    async fn check_verification_status(&self, _verification_id: &str) -> anyhow::Result<KycStatus> {
        // In a real implementation, this would call the KYC provider's API
        // For the mock, we'll randomly return a status
        let statuses = [KycStatus::Pending, KycStatus::Approved, KycStatus::Rejected];
        let random_index = rand::random::<usize>() % statuses.len();
        
        Ok(statuses[random_index].clone())
    }
    
    async fn process_webhook(&self, payload: KycWebhookPayload) -> anyhow::Result<KycVerificationResult> {
        // Parse the status from the payload
        let status = match payload.status.to_lowercase().as_str() {
            "approved" => KycStatus::Approved,
            "rejected" => KycStatus::Rejected,
            _ => KycStatus::Pending,
        };
        
        // Return a mock verification result
        Ok(KycVerificationResult {
            verification_id: payload.verification_id,
            user_id: Uuid::new_v4(), // In a real implementation, we would look up the user ID
            status,
            provider: KycProvider::Internal, // In a real implementation, we would determine the provider
            provider_reference: payload.provider_reference,
            verification_data: payload.data,
            verified_at: payload.timestamp,
        })
    }
    
    async fn get_verification_details(&self, verification_id: &str) -> anyhow::Result<KycVerificationResult> {
        // In a real implementation, this would retrieve data from the database
        Ok(KycVerificationResult {
            verification_id: verification_id.to_string(),
            user_id: Uuid::new_v4(),
            status: KycStatus::Pending,
            provider: KycProvider::Internal,
            provider_reference: format!("mock-ref-{}", verification_id),
            verification_data: serde_json::json!({
                "first_name": "Test",
                "last_name": "User",
                "document_type": "passport",
                "country": "US"
            }),
            verified_at: Utc::now(),
        })
    }
} 