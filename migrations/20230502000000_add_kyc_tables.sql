-- Add KYC provider and level enums
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'kyc_provider') THEN
        CREATE TYPE lsrwa_express.kyc_provider AS ENUM ('internal', 'sumsub', 'onfido', 'shufti', 'persona');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'kyc_level') THEN
        CREATE TYPE lsrwa_express.kyc_level AS ENUM ('basic', 'advanced', 'full');
    END IF;
END$$;

-- KYC verifications table - tracks all KYC verification attempts
CREATE TABLE IF NOT EXISTS lsrwa_express.kyc_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES lsrwa_express.users(id) ON DELETE CASCADE,
    verification_id VARCHAR(255) NOT NULL UNIQUE,
    provider lsrwa_express.kyc_provider NOT NULL DEFAULT 'internal',
    kyc_level lsrwa_express.kyc_level NOT NULL DEFAULT 'basic',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    provider_reference VARCHAR(255),
    verification_url TEXT,
    verification_data JSONB,
    expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMP,
    CONSTRAINT check_kyc_verification_status CHECK (status IN ('pending', 'approved', 'rejected'))
);

-- Create index on user_id for quick lookups
CREATE INDEX IF NOT EXISTS idx_kyc_verifications_user ON lsrwa_express.kyc_verifications(user_id);

-- KYC webhook events table - tracks all incoming webhook events
CREATE TABLE IF NOT EXISTS lsrwa_express.kyc_webhook_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    verification_id VARCHAR(255) NOT NULL,
    provider lsrwa_express.kyc_provider NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    provider_reference VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMP
);

-- Create index on verification_id for quick lookups
CREATE INDEX IF NOT EXISTS idx_kyc_webhook_events_verification ON lsrwa_express.kyc_webhook_events(verification_id);

-- Add KYC provider and level columns to users table
ALTER TABLE lsrwa_express.users
ADD COLUMN IF NOT EXISTS kyc_provider lsrwa_express.kyc_provider,
ADD COLUMN IF NOT EXISTS kyc_level lsrwa_express.kyc_level; 