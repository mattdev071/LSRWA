use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, types::Uuid};

use crate::models::kyc::{
    KycProvider, KycLevel, KycVerificationRequest, KycVerificationResponse, 
    KycVerificationResult, KycWebhookPayload
};
use crate::models::user::KycStatus;

/// Repository for KYC operations
pub struct KycRepository {
    pool: PgPool,
}

impl KycRepository {
    /// Create a new KYC repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Save a new KYC verification request
    pub async fn save_verification(&self, response: &KycVerificationResponse) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO lsrwa_express.kyc_verifications (
                user_id, verification_id, provider, status, verification_url, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            response.user_id,
            response.verification_id,
            response.provider as KycProvider,
            response.status as KycStatus,
            response.verification_url,
            response.expires_at,
        )
        .execute(&self.pool)
        .await
        .context("Failed to save KYC verification")?;
        
        Ok(())
    }
    
    /// Update a KYC verification status
    pub async fn update_verification_status(
        &self, 
        verification_id: &str, 
        status: KycStatus,
        provider_reference: Option<&str>,
        verification_data: Option<serde_json::Value>,
    ) -> Result<()> {
        let verified_at = if status != KycStatus::Pending {
            Some(Utc::now())
        } else {
            None
        };
        
        sqlx::query!(
            r#"
            UPDATE lsrwa_express.kyc_verifications
            SET status = $1, 
                provider_reference = COALESCE($2, provider_reference),
                verification_data = COALESCE($3, verification_data),
                verified_at = $4,
                updated_at = NOW()
            WHERE verification_id = $5
            "#,
            status as KycStatus,
            provider_reference,
            verification_data,
            verified_at,
            verification_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to update KYC verification status")?;
        
        // Also update the user's KYC status
        sqlx::query!(
            r#"
            UPDATE lsrwa_express.users u
            SET kyc_status = $1,
                kyc_timestamp = $2,
                kyc_reference = COALESCE($3, kyc_reference),
                updated_at = NOW()
            FROM lsrwa_express.kyc_verifications kv
            WHERE kv.verification_id = $4
            AND kv.user_id = u.id
            "#,
            status as KycStatus,
            verified_at,
            provider_reference,
            verification_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to update user KYC status")?;
        
        Ok(())
    }
    
    /// Get a KYC verification by ID
    pub async fn get_verification(&self, verification_id: &str) -> Result<Option<KycVerificationResult>> {
        let record = sqlx::query!(
            r#"
            SELECT 
                kv.verification_id,
                kv.user_id,
                kv.status as "status: KycStatus",
                kv.provider as "provider: KycProvider",
                kv.provider_reference,
                kv.verification_data,
                kv.verified_at
            FROM lsrwa_express.kyc_verifications kv
            WHERE kv.verification_id = $1
            "#,
            verification_id,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get KYC verification")?;
        
        Ok(record.map(|r| KycVerificationResult {
            verification_id: r.verification_id,
            user_id: r.user_id,
            status: r.status,
            provider: r.provider,
            provider_reference: r.provider_reference.unwrap_or_default(),
            verification_data: r.verification_data.unwrap_or(serde_json::json!({})),
            verified_at: r.verified_at.unwrap_or_else(Utc::now),
        }))
    }
    
    /// Save a webhook event
    pub async fn save_webhook_event(&self, payload: &KycWebhookPayload) -> Result<Uuid> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO lsrwa_express.kyc_webhook_events (
                verification_id, provider, event_type, status, provider_reference, payload
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            payload.verification_id,
            KycProvider::Internal as KycProvider, // This would be determined based on the payload in production
            payload.event_type,
            payload.status,
            payload.provider_reference,
            payload.data as serde_json::Value,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to save webhook event")?;
        
        Ok(id)
    }
    
    /// Mark a webhook event as processed
    pub async fn mark_webhook_processed(&self, event_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE lsrwa_express.kyc_webhook_events
            SET processed = true, processed_at = NOW()
            WHERE id = $1
            "#,
            event_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to mark webhook as processed")?;
        
        Ok(())
    }
    
    /// Get all verifications for a user
    pub async fn get_user_verifications(&self, user_id: Uuid) -> Result<Vec<KycVerificationResult>> {
        let records = sqlx::query!(
            r#"
            SELECT 
                kv.verification_id,
                kv.user_id,
                kv.status as "status: KycStatus",
                kv.provider as "provider: KycProvider",
                kv.provider_reference,
                kv.verification_data,
                kv.verified_at
            FROM lsrwa_express.kyc_verifications kv
            WHERE kv.user_id = $1
            ORDER BY kv.created_at DESC
            "#,
            user_id,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get user KYC verifications")?;
        
        Ok(records.into_iter().map(|r| KycVerificationResult {
            verification_id: r.verification_id,
            user_id: r.user_id,
            status: r.status,
            provider: r.provider,
            provider_reference: r.provider_reference.unwrap_or_default(),
            verification_data: r.verification_data.unwrap_or(serde_json::json!({})),
            verified_at: r.verified_at.unwrap_or_else(Utc::now),
        }).collect())
    }
} 