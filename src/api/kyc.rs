use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::kyc::{
    KycService, KycVerificationRequest, KycVerificationResponse, 
    KycVerificationResult, KycWebhookPayload, KycProvider
};
use crate::models::user::KycStatus;
use crate::db::kyc_repository::KycRepository;

/// SumSub KYC service implementation
pub struct SumSubKycService {
    client: Client,
    repository: KycRepository,
    base_url: String,
    api_key: String,
    secret_key: String,
}

impl SumSubKycService {
    /// Create a new SumSub KYC service
    pub fn new(pool: PgPool) -> Result<Self> {
        // In production, these would be loaded from environment variables
        let base_url = env::var("SUMSUB_API_URL").unwrap_or_else(|_| "https://api.sumsub.com".to_string());
        let api_key = env::var("SUMSUB_API_KEY").context("SUMSUB_API_KEY must be set")?;
        let secret_key = env::var("SUMSUB_SECRET_KEY").context("SUMSUB_SECRET_KEY must be set")?;
        
        Ok(Self {
            client: Client::new(),
            repository: KycRepository::new(pool),
            base_url,
            api_key,
            secret_key,
        })
    }
    
    /// Generate SumSub authentication token
    fn generate_token(&self) -> String {
        // This is a simplified example - in production, you would use proper HMAC signing
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        format!("{}:{}", self.api_key, timestamp)
    }
}

#[async_trait::async_trait]
impl KycService for SumSubKycService {
    async fn initiate_verification(&self, request: KycVerificationRequest) -> Result<KycVerificationResponse> {
        // Generate a unique external user ID
        let external_user_id = format!("user-{}", request.user_id);
        
        // Create access token for the user
        let token = self.generate_token();
        
        // Create the verification URL
        let level_name = match request.kyc_level {
            crate::models::kyc::KycLevel::Basic => "basic-kyc",
            crate::models::kyc::KycLevel::Advanced => "advanced-kyc",
            crate::models::kyc::KycLevel::Full => "full-kyc",
        };
        
        // In a real implementation, this would call the SumSub API to create an applicant
        // and generate an access token
        
        // For this mock implementation, we'll just create a fake verification URL
        let verification_url = Some(format!(
            "{}/go/verify/{}/{}?level={}&externalUserId={}&accessToken={}",
            self.base_url, request.user_id, uuid::Uuid::new_v4(), level_name, external_user_id, token
        ));
        
        // Create the verification response
        let response = KycVerificationResponse {
            verification_id: uuid::Uuid::new_v4().to_string(),
            user_id: request.user_id,
            status: KycStatus::Pending,
            provider: KycProvider::Sumsub,
            verification_url,
            expires_at: Some(Utc::now() + chrono::Duration::days(7)),
        };
        
        // Save the verification to the database
        self.repository.save_verification(&response).await?;
        
        Ok(response)
    }
    
    async fn check_verification_status(&self, verification_id: &str) -> Result<KycStatus> {
        // In a real implementation, this would call the SumSub API to check the status
        // For this mock implementation, we'll just get the status from our database
        let verification = self.repository.get_verification(verification_id).await?
            .ok_or_else(|| anyhow!("Verification not found"))?;
        
        Ok(verification.status)
    }
    
    async fn process_webhook(&self, payload: KycWebhookPayload) -> Result<KycVerificationResult> {
        // Save the webhook event
        let event_id = self.repository.save_webhook_event(&payload).await?;
        
        // Parse the status from the payload
        let status = match payload.status.to_lowercase().as_str() {
            "approved" | "completed" | "success" => KycStatus::Approved,
            "rejected" | "failed" => KycStatus::Rejected,
            _ => KycStatus::Pending,
        };
        
        // Update the verification status
        self.repository.update_verification_status(
            &payload.verification_id,
            status,
            Some(&payload.provider_reference),
            Some(payload.data.clone()),
        ).await?;
        
        // Mark the webhook as processed
        self.repository.mark_webhook_processed(event_id).await?;
        
        // Return the updated verification
        let verification = self.repository.get_verification(&payload.verification_id).await?
            .ok_or_else(|| anyhow!("Verification not found"))?;
        
        Ok(verification)
    }
    
    async fn get_verification_details(&self, verification_id: &str) -> Result<KycVerificationResult> {
        // In a real implementation, this might call the SumSub API for the latest data
        // For this mock implementation, we'll just get the data from our database
        let verification = self.repository.get_verification(verification_id).await?
            .ok_or_else(|| anyhow!("Verification not found"))?;
        
        Ok(verification)
    }
}

/// Factory for creating KYC services
pub struct KycServiceFactory {
    pool: PgPool,
}

impl KycServiceFactory {
    /// Create a new KYC service factory
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Create a KYC service based on the provider
    pub fn create_service(&self, provider: KycProvider) -> Result<Arc<dyn KycService>> {
        match provider {
            KycProvider::Sumsub => {
                let service = SumSubKycService::new(self.pool.clone())?;
                Ok(Arc::new(service))
            },
            KycProvider::Internal => {
                let service = crate::models::kyc::MockKycService;
                Ok(Arc::new(service))
            },
            _ => Err(anyhow!("KYC provider not implemented: {:?}", provider)),
        }
    }
} 