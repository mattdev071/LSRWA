CREATE SCHEMA IF NOT EXISTS lsrwa_express;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table - stores user information and KYC status
CREATE TABLE lsrwa_express.users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(42) NOT NULL UNIQUE,
    email VARCHAR(255) UNIQUE,
    kyc_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    kyc_timestamp TIMESTAMP,
    kyc_reference VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT check_kyc_status CHECK (kyc_status IN ('pending', 'approved', 'rejected'))
);

-- Create index on wallet address for quick lookups
CREATE INDEX idx_users_wallet ON lsrwa_express.users(wallet_address);

-- Epochs table - tracks epoch lifecycle
CREATE TABLE lsrwa_express.epochs (
    id SERIAL PRIMARY KEY,
    start_timestamp TIMESTAMP NOT NULL,
    end_timestamp TIMESTAMP,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    processed_at TIMESTAMP,
    processing_tx_hash VARCHAR(66),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT check_epoch_status CHECK (status IN ('active', 'processing', 'completed'))
);

-- On-chain request events table - tracks all on-chain request submissions
CREATE TABLE lsrwa_express.blockchain_requests (
    id SERIAL PRIMARY KEY,
    request_type VARCHAR(20) NOT NULL,
    on_chain_id BIGINT NOT NULL,
    wallet_address VARCHAR(42) NOT NULL,
    user_id UUID REFERENCES lsrwa_express.users(id),
    amount NUMERIC(36, 18) NOT NULL,
    collateral_amount NUMERIC(36, 18),
    submission_timestamp TIMESTAMP NOT NULL,
    is_processed BOOLEAN NOT NULL DEFAULT FALSE,
    block_number BIGINT NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT check_request_type CHECK (request_type IN ('deposit', 'withdrawal', 'borrow')),
    CONSTRAINT unique_on_chain_request UNIQUE(request_type, on_chain_id)
);

-- Create index on wallet address and request type for quick lookups
CREATE INDEX idx_blockchain_requests_wallet ON lsrwa_express.blockchain_requests(wallet_address, request_type);
CREATE INDEX idx_blockchain_requests_processed ON lsrwa_express.blockchain_requests(is_processed, request_type);

-- Request processing events - tracks batch processing of requests
CREATE TABLE lsrwa_express.request_processing_events (
    id SERIAL PRIMARY KEY,
    epoch_id INTEGER REFERENCES lsrwa_express.epochs(id),
    processing_type VARCHAR(20) NOT NULL,
    processed_count INTEGER NOT NULL DEFAULT 0,
    transaction_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    processing_timestamp TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT check_processing_type CHECK (processing_type IN ('deposit', 'withdrawal', 'borrow'))
);

-- Request execution events - tracks execution of approved withdrawals
CREATE TABLE lsrwa_express.request_execution_events (
    id SERIAL PRIMARY KEY,
    request_id BIGINT NOT NULL,
    wallet_address VARCHAR(42) NOT NULL,
    amount NUMERIC(36, 18) NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    execution_timestamp TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Batch processing records - tracks which requests were included in each batch
CREATE TABLE lsrwa_express.batch_processing_items (
    id SERIAL PRIMARY KEY,
    processing_event_id INTEGER NOT NULL REFERENCES lsrwa_express.request_processing_events(id),
    request_id BIGINT NOT NULL,
    request_type VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT check_batch_request_type CHECK (request_type IN ('deposit', 'withdrawal', 'borrow')),
    CONSTRAINT check_batch_status CHECK (status IN ('included', 'processed', 'failed'))
);

-- User balances table
CREATE TABLE lsrwa_express.user_balances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE REFERENCES lsrwa_express.users(id) ON DELETE CASCADE,
    active_balance NUMERIC(36, 18) NOT NULL DEFAULT 0,
    pending_deposits NUMERIC(36, 18) NOT NULL DEFAULT 0,
    pending_withdrawals NUMERIC(36, 18) NOT NULL DEFAULT 0,
    total_deposited NUMERIC(36, 18) NOT NULL DEFAULT 0,
    total_withdrawn NUMERIC(36, 18) NOT NULL DEFAULT 0,
    total_rewards NUMERIC(36, 18) NOT NULL DEFAULT 0,
    last_reward_claim_timestamp TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- User rewards table
CREATE TABLE lsrwa_express.user_rewards (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES lsrwa_express.users(id) ON DELETE CASCADE,
    epoch_id INTEGER NOT NULL REFERENCES lsrwa_express.epochs(id),
    amount NUMERIC(36, 18) NOT NULL,
    apr_bps INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    claim_timestamp TIMESTAMP,
    claim_transaction_hash VARCHAR(66),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT check_reward_status CHECK (status IN ('pending', 'claimed', 'expired')),
    CONSTRAINT check_reward_amount CHECK (amount >= 0)
);

-- System parameters table
CREATE TABLE lsrwa_express.system_parameters (
    id SERIAL PRIMARY KEY,
    parameter_name VARCHAR(50) NOT NULL UNIQUE,
    parameter_value TEXT NOT NULL,
    description TEXT,
    updated_by UUID REFERENCES lsrwa_express.users(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Activity logs table
CREATE TABLE lsrwa_express.activity_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES lsrwa_express.users(id),
    activity_type VARCHAR(50) NOT NULL,
    description TEXT,
    data JSONB,
    ip_address VARCHAR(45),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
); 