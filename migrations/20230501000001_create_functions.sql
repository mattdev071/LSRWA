
-- Function to create a new epoch
CREATE OR REPLACE FUNCTION lsrwa_express.create_new_epoch()
RETURNS INTEGER AS $$
DECLARE
    new_epoch_id INTEGER;
BEGIN
    INSERT INTO lsrwa_express.epochs (start_timestamp, status)
    VALUES (NOW(), 'active')
    RETURNING id INTO new_epoch_id;
    
    RETURN new_epoch_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get current active epoch
CREATE OR REPLACE FUNCTION lsrwa_express.get_active_epoch_id()
RETURNS INTEGER AS $$
DECLARE
    active_epoch_id INTEGER;
BEGIN
    SELECT id INTO active_epoch_id
    FROM lsrwa_express.epochs
    WHERE status = 'active'
    ORDER BY id DESC
    LIMIT 1;
    
    RETURN active_epoch_id;
END;
$$ LANGUAGE plpgsql;

-- Function to record an on-chain request
CREATE OR REPLACE FUNCTION lsrwa_express.record_blockchain_request(
    p_request_type VARCHAR, 
    p_on_chain_id BIGINT, 
    p_wallet_address VARCHAR,
    p_amount NUMERIC,
    p_collateral_amount NUMERIC,
    p_timestamp TIMESTAMP,
    p_block_number BIGINT,
    p_tx_hash VARCHAR
)
RETURNS BIGINT AS $$
DECLARE
    user_id UUID;
    request_id BIGINT;
BEGIN
    -- Try to find the user ID
    SELECT id INTO user_id
    FROM lsrwa_express.users
    WHERE wallet_address = p_wallet_address;
    
    -- Insert the request
    INSERT INTO lsrwa_express.blockchain_requests (
        request_type, 
        on_chain_id, 
        wallet_address, 
        user_id, 
        amount, 
        collateral_amount,
        submission_timestamp, 
        is_processed,
        block_number, 
        transaction_hash
    )
    VALUES (
        p_request_type,
        p_on_chain_id,
        p_wallet_address,
        user_id,
        p_amount,
        p_collateral_amount,
        p_timestamp,
        FALSE,
        p_block_number,
        p_tx_hash
    )
    RETURNING id INTO request_id;
    
    RETURN request_id;
END;
$$ LANGUAGE plpgsql;

-- Function to record batch processing event
CREATE OR REPLACE FUNCTION lsrwa_express.record_batch_processing(
    p_epoch_id INTEGER,
    p_processing_type VARCHAR,
    p_request_ids BIGINT[],
    p_tx_hash VARCHAR,
    p_block_number BIGINT,
    p_timestamp TIMESTAMP
)
RETURNS INTEGER AS $$
DECLARE
    processing_id INTEGER;
    request_id BIGINT;
BEGIN
    -- Insert the processing event
    INSERT INTO lsrwa_express.request_processing_events (
        epoch_id,
        processing_type,
        processed_count,
        transaction_hash,
        block_number,
        processing_timestamp
    )
    VALUES (
        p_epoch_id,
        p_processing_type,
        array_length(p_request_ids, 1),
        p_tx_hash,
        p_block_number,
        p_timestamp
    )
    RETURNING id INTO processing_id;
    
    -- Record each processed request
    FOREACH request_id IN ARRAY p_request_ids
    LOOP
        INSERT INTO lsrwa_express.batch_processing_items (
            processing_event_id,
            request_id,
            request_type,
            status
        )
        VALUES (
            processing_id,
            request_id,
            p_processing_type,
            'processed'
        );
        
        -- Update the request status
        UPDATE lsrwa_express.blockchain_requests
        SET is_processed = TRUE
        WHERE request_type = p_processing_type
        AND on_chain_id = request_id;
    END LOOP;
    
    RETURN processing_id;
END;
$$ LANGUAGE plpgsql;

-- Create trigger function to update timestamps
CREATE OR REPLACE FUNCTION lsrwa_express.update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create update triggers for all tables with updated_at column
CREATE TRIGGER update_users_timestamp
BEFORE UPDATE ON lsrwa_express.users
FOR EACH ROW EXECUTE FUNCTION lsrwa_express.update_timestamp();

CREATE TRIGGER update_epochs_timestamp
BEFORE UPDATE ON lsrwa_express.epochs
FOR EACH ROW EXECUTE FUNCTION lsrwa_express.update_timestamp();

CREATE TRIGGER update_blockchain_requests_timestamp
BEFORE UPDATE ON lsrwa_express.blockchain_requests
FOR EACH ROW EXECUTE FUNCTION lsrwa_express.update_timestamp();

CREATE TRIGGER update_request_processing_events_timestamp
BEFORE UPDATE ON lsrwa_express.request_processing_events
FOR EACH ROW EXECUTE FUNCTION lsrwa_express.update_timestamp();

CREATE TRIGGER update_user_balances_timestamp
BEFORE UPDATE ON lsrwa_express.user_balances
FOR EACH ROW EXECUTE FUNCTION lsrwa_express.update_timestamp();

CREATE TRIGGER update_user_rewards_timestamp
BEFORE UPDATE ON lsrwa_express.user_rewards
FOR EACH ROW EXECUTE FUNCTION lsrwa_express.update_timestamp();

CREATE TRIGGER update_system_parameters_timestamp
BEFORE UPDATE ON lsrwa_express.system_parameters
FOR EACH ROW EXECUTE FUNCTION lsrwa_express.update_timestamp(); 