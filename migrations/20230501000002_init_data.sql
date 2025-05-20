INSERT INTO lsrwa_express.system_parameters 
(parameter_name, parameter_value, description) 
VALUES
('reward_apr_bps', '500', 'Default reward APR in basis points (5%)'),
('epoch_duration_seconds', '604800', 'Default epoch duration in seconds (1 week)'),
('max_epochs_before_liquidation', '2', 'Default maximum epochs allowed before liquidation'),
('collateral_ratio_bps', '15000', 'Default collateral ratio in basis points (150%)'),
('min_deposit_amount', '100000000', 'Minimum deposit amount in USDC (100 USDC with 6 decimals)'),
('min_withdrawal_amount', '100000000', 'Minimum withdrawal amount in USDC (100 USDC with 6 decimals)'),
('min_borrow_amount', '1000000000', 'Minimum borrow amount in USDC (1000 USDC with 6 decimals)');

-- Initialize first epoch
SELECT lsrwa_express.create_new_epoch(); 