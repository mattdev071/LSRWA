-- Create the event queue table
CREATE TABLE IF NOT EXISTS lsrwa_express.event_queue (
    id VARCHAR(50) PRIMARY KEY,
    event_type INTEGER NOT NULL,
    block_number BIGINT NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    request_id BIGINT,
    wallet_address VARCHAR(100),
    amount VARCHAR(100),
    request_type VARCHAR(20),
    timestamp TIMESTAMPTZ NOT NULL,
    raw_data TEXT NOT NULL,
    status INTEGER NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_attempt TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on status and attempts for faster queries
CREATE INDEX IF NOT EXISTS event_queue_status_attempts_idx ON lsrwa_express.event_queue (status, attempts);

-- Create index on block_number for faster queries
CREATE INDEX IF NOT EXISTS event_queue_block_number_idx ON lsrwa_express.event_queue (block_number);

-- Create index on transaction_hash for faster queries
CREATE INDEX IF NOT EXISTS event_queue_transaction_hash_idx ON lsrwa_express.event_queue (transaction_hash);

-- Create index on request_id for faster queries
CREATE INDEX IF NOT EXISTS event_queue_request_id_idx ON lsrwa_express.event_queue (request_id);

-- Create index on wallet_address for faster queries
CREATE INDEX IF NOT EXISTS event_queue_wallet_address_idx ON lsrwa_express.event_queue (wallet_address);

-- Create the system settings table
CREATE TABLE IF NOT EXISTS lsrwa_express.system_settings (
    key VARCHAR(50) PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION lsrwa_express.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger for the event_queue table
CREATE TRIGGER update_event_queue_updated_at
BEFORE UPDATE ON lsrwa_express.event_queue
FOR EACH ROW
EXECUTE FUNCTION lsrwa_express.update_updated_at_column();

-- Create a trigger for the system_settings table
CREATE TRIGGER update_system_settings_updated_at
BEFORE UPDATE ON lsrwa_express.system_settings
FOR EACH ROW
EXECUTE FUNCTION lsrwa_express.update_updated_at_column(); 