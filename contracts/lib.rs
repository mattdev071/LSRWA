#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod lsrwa_express {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    /// Request types enum - mirrors the backend RequestType enum
    #[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum RequestType {
        Deposit,
        Withdrawal,
        Borrow,
    }

    /// Request validation result enum
    #[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum ValidationResult {
        Valid,
        InvalidAmount,
        InsufficientBalance,
        UserNotRegistered,
        DailyLimitExceeded,
        RequestLimitExceeded,
        ValidationError,
    }

    /// Batch item status enum - mirrors the backend BatchItemStatus enum
    #[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum BatchItemStatus {
        Included,
        Processed,
        Failed,
    }

    /// Request data structure - mirrors the backend BlockchainRequest model
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Request {
        id: u128,                    // On-chain unique identifier
        request_type: RequestType,   // Type of request
        wallet_address: AccountId,   // User's wallet address
        amount: Balance,             // Request amount
        collateral_amount: Option<Balance>, // Collateral amount (for borrow requests)
        timestamp: Timestamp,        // Submission timestamp
        is_processed: bool,          // Whether the request has been processed
        block_number: BlockNumber,   // Block number when the request was submitted
        transaction_hash: Hash,      // Transaction hash of the request
    }

    /// Processing event data structure - mirrors the backend RequestProcessingEvent
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct ProcessingEvent {
        id: u128,                    // On-chain unique identifier
        epoch_id: u128,              // Associated epoch ID
        processing_type: RequestType, // Type of processing
        request_ids: Vec<u128>,      // IDs of the processed requests
        processed_count: u128,       // Number of processed requests
        transaction_hash: Hash,      // Transaction hash of the processing
        block_number: BlockNumber,   // Block number when the processing occurred
        timestamp: Timestamp,        // Processing timestamp
    }

    /// Execution event data structure - mirrors the backend RequestExecutionEvent
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct ExecutionEvent {
        id: u128,                    // On-chain unique identifier
        request_id: u128,            // Associated request ID
        wallet_address: AccountId,   // User's wallet address
        amount: Balance,             // Executed amount
        transaction_hash: Hash,      // Transaction hash of the execution
        block_number: BlockNumber,   // Block number when the execution occurred
        timestamp: Timestamp,        // Execution timestamp
    }

    /// Epoch data structure - mirrors the backend Epoch model
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Epoch {
        id: u128,                    // On-chain unique identifier
        start_timestamp: Timestamp,  // Start timestamp
        end_timestamp: Option<Timestamp>, // End timestamp (None if active)
        is_active: bool,             // Whether the epoch is active
        processed_at: Option<Timestamp>, // Processing timestamp (None if not processed)
        processing_tx_hash: Option<Hash>, // Transaction hash of processing (None if not processed)
    }

    /// User data structure - mirrors the backend User model with balance
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct User {
        wallet_address: AccountId,   // User's wallet address
        is_registered: bool,         // Whether the user is registered
        active_balance: Balance,     // Active balance
        pending_deposits: Balance,   // Pending deposits amount
        pending_withdrawals: Balance, // Pending withdrawals amount
        total_rewards: Balance,      // Total accumulated rewards
        registered_at: Timestamp,    // Registration timestamp
    }

    /// Event emitted when a deposit is requested
    #[ink(event)]
    pub struct DepositRequested {
        #[ink(topic)]
        request_id: u128,
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
        timestamp: Timestamp,
    }

    /// Event emitted when a withdrawal is requested
    #[ink(event)]
    pub struct WithdrawalRequested {
        #[ink(topic)]
        request_id: u128,
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
        timestamp: Timestamp,
    }

    /// Event emitted when a borrow is requested
    #[ink(event)]
    pub struct BorrowRequested {
        #[ink(topic)]
        request_id: u128,
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
        collateral_amount: Balance,
        timestamp: Timestamp,
    }

    /// Event emitted when a batch is processed
    #[ink(event)]
    pub struct BatchProcessed {
        #[ink(topic)]
        batch_id: u128,
        #[ink(topic)]
        request_type: RequestType,
        processed_count: u128,
        timestamp: Timestamp,
    }

    /// Event emitted when a request is executed
    #[ink(event)]
    pub struct RequestExecuted {
        #[ink(topic)]
        request_id: u128,
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
        timestamp: Timestamp,
    }

    /// Event emitted when an epoch is created
    #[ink(event)]
    pub struct EpochCreated {
        #[ink(topic)]
        epoch_id: u128,
        start_timestamp: Timestamp,
    }

    /// Event emitted when an epoch is closed
    #[ink(event)]
    pub struct EpochClosed {
        #[ink(topic)]
        epoch_id: u128,
        end_timestamp: Timestamp,
    }

    /// Event emitted when a user is registered
    #[ink(event)]
    pub struct UserRegistered {
        #[ink(topic)]
        wallet_address: AccountId,
        timestamp: Timestamp,
    }

    /// Event emitted when a request validation fails
    #[ink(event)]
    pub struct RequestValidationFailed {
        #[ink(topic)]
        wallet_address: AccountId,
        request_type: RequestType,
        amount: Balance,
        #[ink(topic)]
        reason: ValidationResult,
        timestamp: Timestamp,
    }

    /// Lsrwa Express contract storage
    #[ink(storage)]
    pub struct LsrwaExpress {
        /// Contract owner
        owner: AccountId,
        
        /// Contract admin
        admin: AccountId,
        
        /// Current epoch ID
        current_epoch_id: u128,
        
        /// Next request ID
        next_request_id: u128,
        
        /// Next processing event ID
        next_processing_event_id: u128,
        
        /// Next execution event ID
        next_execution_event_id: u128,
        
        /// Mapping from request ID to Request
        requests: Mapping<u128, Request>,
        
        /// Mapping from request type and counter to request ID
        request_ids_by_type: Mapping<(RequestType, u128), u128>,
        
        /// Mapping from request type to count
        request_count_by_type: Mapping<RequestType, u128>,
        
        /// Mapping from wallet address to deposit request IDs
        user_deposit_requests: Mapping<AccountId, Vec<u128>>,
        
        /// Mapping from wallet address to withdrawal request IDs
        user_withdrawal_requests: Mapping<AccountId, Vec<u128>>,
        
        /// Mapping from wallet address to borrow request IDs
        user_borrow_requests: Mapping<AccountId, Vec<u128>>,
        
        /// Mapping from processing event ID to ProcessingEvent
        processing_events: Mapping<u128, ProcessingEvent>,
        
        /// Mapping from execution event ID to ExecutionEvent
        execution_events: Mapping<u128, ExecutionEvent>,
        
        /// Mapping from epoch ID to Epoch
        epochs: Mapping<u128, Epoch>,
        
        /// Mapping from wallet address to User
        users: Mapping<AccountId, User>,
        
        /// Mapping from processing event ID and request ID to status
        batch_items: Mapping<(u128, u128), BatchItemStatus>,
        
        /// Daily withdrawal limit per user
        daily_withdrawal_limit: Balance,
        
        /// Maximum number of pending requests per user
        max_pending_requests: u32,
        
        /// Minimum deposit amount
        min_deposit_amount: Balance,
        
        /// Minimum withdrawal amount
        min_withdrawal_amount: Balance,
        
        /// Mapping from wallet address to timestamp of last daily limit reset
        last_limit_reset: Mapping<AccountId, Timestamp>,
        
        /// Mapping from wallet address to current daily withdrawal total
        daily_withdrawal_total: Mapping<AccountId, Balance>,
    }

    impl LsrwaExpress {
        /// Constructor that initializes the contract with the caller as the owner
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            let current_time = Self::env().block_timestamp();
            
            // Create initial epoch
            let epoch_id = 1;
            let epoch = Epoch {
                id: epoch_id,
                start_timestamp: current_time,
                end_timestamp: None,
                is_active: true,
                processed_at: None,
                processing_tx_hash: None,
            };
            
            // Initialize the contract
            let mut this = Self {
                owner: caller,
                admin: caller,
                current_epoch_id: epoch_id,
                next_request_id: 1,
                next_processing_event_id: 1,
                next_execution_event_id: 1,
                requests: Mapping::default(),
                request_ids_by_type: Mapping::default(),
                request_count_by_type: Mapping::default(),
                user_deposit_requests: Mapping::default(),
                user_withdrawal_requests: Mapping::default(),
                user_borrow_requests: Mapping::default(),
                processing_events: Mapping::default(),
                execution_events: Mapping::default(),
                epochs: Mapping::default(),
                users: Mapping::default(),
                batch_items: Mapping::default(),
                daily_withdrawal_limit: 10_000, // 10,000 tokens per day
                max_pending_requests: 5,        // Maximum 5 pending requests per user
                min_deposit_amount: 10,         // Minimum 10 tokens for deposit
                min_withdrawal_amount: 10,      // Minimum 10 tokens for withdrawal
                last_limit_reset: Mapping::default(),
                daily_withdrawal_total: Mapping::default(),
            };
            
            // Store the initial epoch
            this.epochs.insert(epoch_id, &epoch);
            
            // Emit epoch created event
            Self::env().emit_event(EpochCreated {
                epoch_id,
                start_timestamp: current_time,
            });
            
            this
        }
        
        /// Returns the owner of the contract
        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner
        }
        
        /// Returns the current epoch ID
        #[ink(message)]
        pub fn get_current_epoch_id(&self) -> u128 {
            self.current_epoch_id
        }
        
        /// Returns the epoch with the given ID
        #[ink(message)]
        pub fn get_epoch(&self, epoch_id: u128) -> Option<Epoch> {
            self.epochs.get(epoch_id)
        }
        
        /// Returns the request count for the given type
        #[ink(message)]
        pub fn get_request_count(&self, request_type: RequestType) -> u128 {
            self.request_count_by_type.get(request_type).unwrap_or(0)
        }
        
        /// Returns the request with the given ID
        #[ink(message)]
        pub fn get_request(&self, request_id: u128) -> Option<Request> {
            self.requests.get(request_id)
        }
        
        /// Returns the user with the given wallet address
        #[ink(message)]
        pub fn get_user(&self, wallet_address: AccountId) -> Option<User> {
            self.users.get(wallet_address)
        }
        
        /// Updates the daily withdrawal limit
        #[ink(message)]
        pub fn set_daily_withdrawal_limit(&mut self, new_limit: Balance) -> Result<(), &'static str> {
            // Only owner can update limits
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err("Only owner can update limits");
            }
            
            self.daily_withdrawal_limit = new_limit;
            Ok(())
        }
        
        /// Updates the maximum number of pending requests per user
        #[ink(message)]
        pub fn set_max_pending_requests(&mut self, new_max: u32) -> Result<(), &'static str> {
            // Only owner can update limits
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err("Only owner can update limits");
            }
            
            self.max_pending_requests = new_max;
            Ok(())
        }
        
        /// Updates the minimum deposit amount
        #[ink(message)]
        pub fn set_min_deposit_amount(&mut self, new_min: Balance) -> Result<(), &'static str> {
            // Only owner can update limits
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err("Only owner can update limits");
            }
            
            self.min_deposit_amount = new_min;
            Ok(())
        }
        
        /// Updates the minimum withdrawal amount
        #[ink(message)]
        pub fn set_min_withdrawal_amount(&mut self, new_min: Balance) -> Result<(), &'static str> {
            // Only owner can update limits
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err("Only owner can update limits");
            }
            
            self.min_withdrawal_amount = new_min;
            Ok(())
        }
        
        /// Validates a deposit request
        fn validate_deposit_request(&self, caller: AccountId, amount: Balance) -> ValidationResult {
            // Check minimum amount
            if amount < self.min_deposit_amount {
                return ValidationResult::InvalidAmount;
            }
            
            // Check if user has too many pending requests
            let user_deposits = self.user_deposit_requests.get(caller).unwrap_or_default();
            let mut pending_count = 0;
            
            for request_id in user_deposits.iter() {
                if let Some(request) = self.requests.get(request_id) {
                    if !request.is_processed {
                        pending_count += 1;
                    }
                }
            }
            
            if pending_count as u32 >= self.max_pending_requests {
                return ValidationResult::RequestLimitExceeded;
            }
            
            ValidationResult::Valid
        }
        
        /// Validates a withdrawal request
        fn validate_withdrawal_request(&mut self, caller: AccountId, amount: Balance) -> ValidationResult {
            // Check if the user exists and is registered
            let user = match self.users.get(caller) {
                Some(user) => user,
                None => return ValidationResult::UserNotRegistered,
            };
            
            if !user.is_registered {
                return ValidationResult::UserNotRegistered;
            }
            
            // Check minimum amount
            if amount < self.min_withdrawal_amount {
                return ValidationResult::InvalidAmount;
            }
            
            // Check if user has sufficient balance
            if user.active_balance < amount {
                return ValidationResult::InsufficientBalance;
            }
            
            // Check if user has too many pending requests
            let user_withdrawals = self.user_withdrawal_requests.get(caller).unwrap_or_default();
            let mut pending_count = 0;
            
            for request_id in user_withdrawals.iter() {
                if let Some(request) = self.requests.get(request_id) {
                    if !request.is_processed {
                        pending_count += 1;
                    }
                }
            }
            
            if pending_count as u32 >= self.max_pending_requests {
                return ValidationResult::RequestLimitExceeded;
            }
            
            // Check daily withdrawal limit
            let current_time = Self::env().block_timestamp();
            let last_reset = self.last_limit_reset.get(caller).unwrap_or(0);
            let one_day = 86400000; // 24 hours in milliseconds
            
            let daily_total = if current_time > last_reset + one_day {
                // Reset daily total if it's been more than a day
                self.last_limit_reset.insert(caller, &current_time);
                self.daily_withdrawal_total.insert(caller, &amount);
                amount
            } else {
                // Add to daily total
                let current_total = self.daily_withdrawal_total.get(caller).unwrap_or(0);
                let new_total = current_total + amount;
                self.daily_withdrawal_total.insert(caller, &new_total);
                new_total
            };
            
            if daily_total > self.daily_withdrawal_limit {
                // Rollback the daily total update
                self.daily_withdrawal_total.insert(caller, &(daily_total - amount));
                return ValidationResult::DailyLimitExceeded;
            }
            
            ValidationResult::Valid
        }

        /// Creates a deposit request for the caller
        #[ink(message)]
        pub fn create_deposit_request(&mut self, amount: Balance) -> Result<u128, &'static str> {
            // Get the caller's wallet address
            let caller = Self::env().caller();
            
            // Ensure amount is greater than zero
            if amount == 0 {
                return Err("Amount must be greater than zero");
            }
            
            // Validate the deposit request
            let validation_result = self.validate_deposit_request(caller, amount);
            if validation_result != ValidationResult::Valid {
                let current_time = Self::env().block_timestamp();
                
                // Emit validation failure event
                Self::env().emit_event(RequestValidationFailed {
                    wallet_address: caller,
                    request_type: RequestType::Deposit,
                    amount,
                    reason: validation_result,
                    timestamp: current_time,
                });
                
                return match validation_result {
                    ValidationResult::InvalidAmount => Err("Amount is below minimum deposit amount"),
                    ValidationResult::RequestLimitExceeded => Err("Too many pending requests"),
                    _ => Err("Validation failed"),
                };
            }
            
            // Check if the user exists, if not, register them
            let user = self.users.get(caller);
            if user.is_none() {
                let current_time = Self::env().block_timestamp();
                let new_user = User {
                    wallet_address: caller,
                    is_registered: true, // Auto-register the user
                    active_balance: 0,
                    pending_deposits: 0,
                    pending_withdrawals: 0,
                    total_rewards: 0,
                    registered_at: current_time,
                };
                
                // Store the new user
                self.users.insert(caller, &new_user);
                
                // Emit user registered event
                Self::env().emit_event(UserRegistered {
                    wallet_address: caller,
                    timestamp: current_time,
                });
            }
            
            // Get current request ID and increment for next use
            let request_id = self.next_request_id;
            self.next_request_id += 1;
            
            // Get current timestamp and block number
            let current_time = Self::env().block_timestamp();
            let block_number = Self::env().block_number();
            
            // For testnet readiness, we use the current transaction hash
            // This will give us the actual transaction hash of the call to this function
            let tx_hash = Self::env().transaction_hash().unwrap_or_else(|| {
                // Fallback for cases where the transaction hash is not available (e.g. in tests)
                Self::env().hash_bytes::<ink::env::hash::Blake2x256>(
                    &[current_time.to_be_bytes(), block_number.to_be_bytes(), caller.as_ref()].concat()
                )
            });
            
            // Create the deposit request
            let request = Request {
                id: request_id,
                request_type: RequestType::Deposit,
                wallet_address: caller,
                amount,
                collateral_amount: None, // Not applicable for deposits
                timestamp: current_time,
                is_processed: false,
                block_number,
                transaction_hash: tx_hash,
            };
            
            // Store the request
            self.requests.insert(request_id, &request);
            
            // Update request count for deposit type
            let current_count = self.get_request_count(RequestType::Deposit);
            self.request_count_by_type.insert(RequestType::Deposit, &(current_count + 1));
            
            // Store the request ID in the type-indexed mapping
            self.request_ids_by_type.insert((RequestType::Deposit, current_count), &request_id);
            
            // Add the request ID to the user's deposit requests
            let mut user_deposits = self.user_deposit_requests.get(caller).unwrap_or_default();
            user_deposits.push(request_id);
            self.user_deposit_requests.insert(caller, &user_deposits);
            
            // Update user's pending deposits
            if let Some(mut user) = self.users.get(caller) {
                user.pending_deposits += amount;
                self.users.insert(caller, &user);
            }
            
            // Emit deposit requested event
            Self::env().emit_event(DepositRequested {
                request_id,
                wallet_address: caller,
                amount,
                timestamp: current_time,
            });
            
            Ok(request_id)
        }
        
        /// Creates a withdrawal request for the caller
        #[ink(message)]
        pub fn create_withdrawal_request(&mut self, amount: Balance) -> Result<u128, &'static str> {
            // Get the caller's wallet address
            let caller = Self::env().caller();
            
            // Ensure amount is greater than zero
            if amount == 0 {
                return Err("Amount must be greater than zero");
            }
            
            // Validate the withdrawal request
            let validation_result = self.validate_withdrawal_request(caller, amount);
            if validation_result != ValidationResult::Valid {
                let current_time = Self::env().block_timestamp();
                
                // Emit validation failure event
                Self::env().emit_event(RequestValidationFailed {
                    wallet_address: caller,
                    request_type: RequestType::Withdrawal,
                    amount,
                    reason: validation_result,
                    timestamp: current_time,
                });
                
                return match validation_result {
                    ValidationResult::UserNotRegistered => Err("User not registered"),
                    ValidationResult::InvalidAmount => Err("Amount is below minimum withdrawal amount"),
                    ValidationResult::InsufficientBalance => Err("Insufficient balance for withdrawal"),
                    ValidationResult::RequestLimitExceeded => Err("Too many pending requests"),
                    ValidationResult::DailyLimitExceeded => Err("Daily withdrawal limit exceeded"),
                    _ => Err("Validation failed"),
                };
            }
            
            // Get current request ID and increment for next use
            let request_id = self.next_request_id;
            self.next_request_id += 1;
            
            // Get current timestamp and block number
            let current_time = Self::env().block_timestamp();
            let block_number = Self::env().block_number();
            
            // For testnet readiness, we use the current transaction hash
            // This will give us the actual transaction hash of the call to this function
            let tx_hash = Self::env().transaction_hash().unwrap_or_else(|| {
                // Fallback for cases where the transaction hash is not available (e.g. in tests)
                Self::env().hash_bytes::<ink::env::hash::Blake2x256>(
                    &[current_time.to_be_bytes(), block_number.to_be_bytes(), caller.as_ref()].concat()
                )
            });
            
            // Create the withdrawal request
            let request = Request {
                id: request_id,
                request_type: RequestType::Withdrawal,
                wallet_address: caller,
                amount,
                collateral_amount: None, // Not applicable for withdrawals
                timestamp: current_time,
                is_processed: false,
                block_number,
                transaction_hash: tx_hash,
            };
            
            // Store the request
            self.requests.insert(request_id, &request);
            
            // Update request count for withdrawal type
            let current_count = self.get_request_count(RequestType::Withdrawal);
            self.request_count_by_type.insert(RequestType::Withdrawal, &(current_count + 1));
            
            // Store the request ID in the type-indexed mapping
            self.request_ids_by_type.insert((RequestType::Withdrawal, current_count), &request_id);
            
            // Add the request ID to the user's withdrawal requests
            let mut user_withdrawals = self.user_withdrawal_requests.get(caller).unwrap_or_default();
            user_withdrawals.push(request_id);
            self.user_withdrawal_requests.insert(caller, &user_withdrawals);
            
            // Update user's balances
            if let Some(mut user) = self.users.get(caller) {
                user.active_balance -= amount;
                user.pending_withdrawals += amount;
                self.users.insert(caller, &user);
            }
            
            // Emit withdrawal requested event
            Self::env().emit_event(WithdrawalRequested {
                request_id,
                wallet_address: caller,
                amount,
                timestamp: current_time,
            });
            
            Ok(request_id)
        }
        
        /// Checks if a user is registered
        #[ink(message)]
        pub fn is_registered(&self, wallet_address: AccountId) -> bool {
            if let Some(user) = self.users.get(wallet_address) {
                user.is_registered
            } else {
                false
            }
        }
        
        /// Updates a user's registration status
        /// Can only be called by the contract owner or admin
        #[ink(message)]
        pub fn update_registration_status(&mut self, wallet_address: AccountId, is_approved: bool) -> Result<(), &'static str> {
            // Only owner or admin can update registration status
            let caller = Self::env().caller();
            if caller != self.owner && caller != self.admin {
                return Err("Only owner or admin can update registration status");
            }
            
            // Check if the user exists
            let mut user = self.users.get(wallet_address).ok_or("User not found")?;
            
            // Update registration status
            user.is_registered = is_approved;
            
            // Store the updated user
            self.users.insert(wallet_address, &user);
            
            // Emit user registered event if approved
            if is_approved {
                Self::env().emit_event(UserRegistered {
                    wallet_address,
                    timestamp: Self::env().block_timestamp(),
                });
            }
            
            Ok(())
        }
        
        /// Process a deposit request
        /// Can only be called by the contract owner or admin
        #[ink(message)]
        pub fn process_deposit_request(&mut self, request_id: u128) -> Result<(), &'static str> {
            // Only owner or admin can process requests
            let caller = Self::env().caller();
            if caller != self.owner && caller != self.admin {
                return Err("Only owner or admin can process requests");
            }
            
            // Get the request
            let mut request = self.requests.get(request_id).ok_or("Request not found")?;
            
            // Ensure the request is a deposit
            if request.request_type != RequestType::Deposit {
                return Err("Not a deposit request");
            }
            
            // Ensure the request is not already processed
            if request.is_processed {
                return Err("Request already processed");
            }
            
            // Get the user
            let mut user = self.users.get(request.wallet_address).ok_or("User not found")?;
            
            // Check if the user is registered
            if !user.is_registered {
                return Err("User not registered");
            }
            
            // Update the user's balances
            user.active_balance += request.amount;
            user.pending_deposits -= request.amount;
            
            // Mark the request as processed
            request.is_processed = true;
            
            // Store the updated user and request
            self.users.insert(request.wallet_address, &user);
            self.requests.insert(request_id, &request);
            
            // Create an execution event
            let execution_id = self.next_execution_event_id;
            self.next_execution_event_id += 1;
            
            let current_time = Self::env().block_timestamp();
            let block_number = Self::env().block_number();
            
            // For testnet readiness, we use the current transaction hash
            let tx_hash = Self::env().transaction_hash().unwrap_or_else(|| {
                // Fallback for cases where the transaction hash is not available (e.g. in tests)
                Self::env().hash_bytes::<ink::env::hash::Blake2x256>(
                    &[current_time.to_be_bytes(), block_number.to_be_bytes(), caller.as_ref()].concat()
                )
            });
            
            let execution_event = ExecutionEvent {
                id: execution_id,
                request_id,
                wallet_address: request.wallet_address,
                amount: request.amount,
                transaction_hash: tx_hash,
                block_number,
                timestamp: current_time,
            };
            
            // Store the execution event
            self.execution_events.insert(execution_id, &execution_event);
            
            // Emit request executed event
            Self::env().emit_event(RequestExecuted {
                request_id,
                wallet_address: request.wallet_address,
                amount: request.amount,
                timestamp: current_time,
            });
            
            Ok(())
        }
        
        /// Processes a withdrawal request, marking it as processed and updating balances
        #[ink(message)]
        pub fn process_withdrawal_request(&mut self, request_id: u128) -> Result<(), &'static str> {
            // Only owner or admin can process requests
            let caller = Self::env().caller();
            if caller != self.owner && caller != self.admin {
                return Err("Only owner or admin can process requests");
            }
            
            // Get the request
            let mut request = self.requests.get(request_id).ok_or("Request not found")?;
            
            // Ensure the request is a withdrawal
            if request.request_type != RequestType::Withdrawal {
                return Err("Not a withdrawal request");
            }
            
            // Ensure the request is not already processed
            if request.is_processed {
                return Err("Request already processed");
            }
            
            // Get the user
            let mut user = self.users.get(request.wallet_address).ok_or("User not found")?;
            
            // Check if the user is registered
            if !user.is_registered {
                return Err("User not registered");
            }
            
            // Update the user's balances - reduce pending withdrawals
            // Note: active_balance was already reduced when creating the withdrawal request
            user.pending_withdrawals -= request.amount;
            
            // Mark the request as processed
            request.is_processed = true;
            
            // Store the updated user and request
            self.users.insert(request.wallet_address, &user);
            self.requests.insert(request_id, &request);
            
            // Create an execution event
            let execution_id = self.next_execution_event_id;
            self.next_execution_event_id += 1;
            
            let current_time = Self::env().block_timestamp();
            let block_number = Self::env().block_number();
            
            // For testnet readiness, we use the current transaction hash
            let tx_hash = Self::env().transaction_hash().unwrap_or_else(|| {
                // Fallback for cases where the transaction hash is not available
                Self::env().hash_bytes::<ink::env::hash::Blake2x256>(
                    &[current_time.to_be_bytes(), block_number.to_be_bytes(), request.wallet_address.as_ref()].concat()
                )
            });
            
            // Create and store the execution event
            let execution_event = ExecutionEvent {
                id: execution_id,
                request_id,
                wallet_address: request.wallet_address,
                amount: request.amount,
                transaction_hash: tx_hash,
                block_number,
                timestamp: current_time,
            };
            self.execution_events.insert(execution_id, &execution_event);
            
            // Emit request executed event
            Self::env().emit_event(RequestExecuted {
                request_id,
                wallet_address: request.wallet_address,
                amount: request.amount,
                timestamp: current_time,
            });
            
            Ok(())
        }
    }
    
    /// Unit tests for the contract
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};
        use ink::env::test::DefaultAccounts;

        type Env = DefaultEnvironment;
        
        /// Helper function to get default accounts for testing
        fn get_default_accounts() -> DefaultAccounts<Env> {
            test::default_accounts::<Env>()
        }
        
        /// Helper function to initialize the contract for testing
        fn init_contract() -> LsrwaExpress {
            let accounts = get_default_accounts();
            
            // Set the contract call as coming from account 0
            test::set_caller::<Env>(accounts.alice);
            
            // Create a new contract instance
            LsrwaExpress::new()
        }
        
        /// Test the contract initialization
        #[ink::test]
        fn test_init() {
            let accounts = get_default_accounts();
            let contract = init_contract();
            
            // Test that the owner is set to the caller
            assert_eq!(contract.get_owner(), accounts.alice);
            
            // Test that the current epoch ID is set to 1
            assert_eq!(contract.get_current_epoch_id(), 1);
            
            // Test that the first epoch is created
            let epoch = contract.get_epoch(1).expect("Epoch 1 should exist");
            assert_eq!(epoch.id, 1);
            assert!(epoch.is_active);
            assert!(epoch.end_timestamp.is_none());
            assert!(epoch.processed_at.is_none());
            assert!(epoch.processing_tx_hash.is_none());
            
            // Test that the request count is 0 for all types
            assert_eq!(contract.get_request_count(RequestType::Deposit), 0);
            assert_eq!(contract.get_request_count(RequestType::Withdrawal), 0);
            assert_eq!(contract.get_request_count(RequestType::Borrow), 0);
        }
        
        /// Test getting non-existent objects
        #[ink::test]
        fn test_get_nonexistent() {
            let accounts = get_default_accounts();
            let contract = init_contract();
            
            // Test that getting a non-existent epoch returns None
            assert!(contract.get_epoch(2).is_none());
            
            // Test that getting a non-existent request returns None
            assert!(contract.get_request(1).is_none());
            
            // Test that getting a non-existent user returns None
            assert!(contract.get_user(accounts.bob).is_none());
        }
        
        /// Test creating a deposit request
        #[ink::test]
        fn test_create_deposit_request() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for this test
            test::set_caller::<Env>(accounts.bob);
            
            // Create a deposit request
            let deposit_amount = 1000;
            let request_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Verify the request ID is 1
            assert_eq!(request_id, 1);
            
            // Verify the request count is updated
            assert_eq!(contract.get_request_count(RequestType::Deposit), 1);
            
            // Verify the request exists and has the correct data
            let request = contract.get_request(request_id).expect("Request should exist");
            assert_eq!(request.id, request_id);
            assert_eq!(request.request_type, RequestType::Deposit);
            assert_eq!(request.wallet_address, accounts.bob);
            assert_eq!(request.amount, deposit_amount);
            assert_eq!(request.collateral_amount, None);
            assert!(!request.is_processed);
            
            // Verify the user was created and automatically registered
            let user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(user.wallet_address, accounts.bob);
            assert!(user.is_registered);
            assert_eq!(user.pending_deposits, deposit_amount);
            assert_eq!(user.active_balance, 0);
            
            // Create another deposit request
            let second_deposit_amount = 500;
            let second_request_id = contract.create_deposit_request(second_deposit_amount).expect("Should create second deposit request");
            
            // Verify the request ID is 2
            assert_eq!(second_request_id, 2);
            
            // Verify the request count is updated
            assert_eq!(contract.get_request_count(RequestType::Deposit), 2);
            
            // Verify the user's pending deposits are updated
            let updated_user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(updated_user.pending_deposits, deposit_amount + second_deposit_amount);
        }
        
        /// Test creating a deposit request with zero amount
        #[ink::test]
        fn test_create_deposit_request_zero_amount() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for this test
            test::set_caller::<Env>(accounts.bob);
            
            // Try to create a deposit request with zero amount
            let deposit_amount = 0;
            let result = contract.create_deposit_request(deposit_amount);
            
            // Verify the request fails with the expected error
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Amount must be greater than zero");
            
            // Verify no request was created
            assert_eq!(contract.get_request_count(RequestType::Deposit), 0);
            
            // Verify no user was created
            assert!(contract.get_user(accounts.bob).is_none());
        }
        
        /// Test processing a deposit request
        #[ink::test]
        fn test_process_deposit_request() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for registration
            test::set_caller::<Env>(accounts.bob);
            
            // Create a deposit request (which automatically registers the user)
            let deposit_amount = 1000;
            let request_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Verify the user is registered
            let user = contract.get_user(accounts.bob).expect("User should exist");
            assert!(user.is_registered);
            
            // Set the caller back to Alice (owner) to process the deposit
            test::set_caller::<Env>(accounts.alice);
            
            // Process the deposit request
            contract.process_deposit_request(request_id).expect("Should process deposit request");
            
            // Verify the request is now processed
            let request = contract.get_request(request_id).expect("Request should exist");
            assert!(request.is_processed);
            
            // Verify the user's balances are updated
            let user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(user.active_balance, deposit_amount);
            assert_eq!(user.pending_deposits, 0);
        }
        
        /// Test registration approval process
        #[ink::test]
        fn test_registration_approval() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for registration
            test::set_caller::<Env>(accounts.bob);
            
            // Create a deposit request to register the user
            let deposit_amount = 1000;
            let request_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Verify the user exists but is not registered yet
            let user = contract.get_user(accounts.bob).expect("User should exist");
            assert!(!user.is_registered);
            
            // Set the caller back to Alice (owner) to approve registration
            test::set_caller::<Env>(accounts.alice);
            
            // Approve Bob's registration
            contract.update_registration_status(accounts.bob, true).expect("Should update registration status");
            
            // Verify the user is now registered
            let updated_user = contract.get_user(accounts.bob).expect("User should exist");
            assert!(updated_user.is_registered);
            
            // Test the is_registered function
            assert!(contract.is_registered(accounts.bob));
            
            // Test registration disapproval
            contract.update_registration_status(accounts.bob, false).expect("Should update registration status");
            
            // Verify the user is now unregistered
            let disapproved_user = contract.get_user(accounts.bob).expect("User should exist");
            assert!(!disapproved_user.is_registered);
            
            // Test the is_registered function again
            assert!(!contract.is_registered(accounts.bob));
        }
        
        /// Test non-admin/owner trying to update registration or process deposit
        #[ink::test]
        fn test_unauthorized_actions() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for registration
            test::set_caller::<Env>(accounts.bob);
            
            // Create a deposit request
            let deposit_amount = 1000;
            let request_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Try to update registration status (still as Bob)
            let registration_result = contract.update_registration_status(accounts.bob, true);
            
            // Verify the update fails with the expected error
            assert!(registration_result.is_err());
            assert_eq!(registration_result.unwrap_err(), "Only owner or admin can update registration status");
            
            // Try to process the deposit request (still as Bob)
            let process_result = contract.process_deposit_request(request_id);
            
            // Verify the processing fails with the expected error
            assert!(process_result.is_err());
            assert_eq!(process_result.unwrap_err(), "Only owner or admin can process requests");
        }
        
        /// Test creating and processing a withdrawal request
        #[ink::test]
        fn test_withdrawal_request() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for this test
            test::set_caller::<Env>(accounts.bob);
            
            // First create a deposit to have funds
            let deposit_amount = 1000;
            let deposit_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Process the deposit as admin to make funds available
            test::set_caller::<Env>(accounts.alice); // Admin/owner
            contract.process_deposit_request(deposit_id).expect("Should process deposit");
            
            // Now create a withdrawal request as Bob
            test::set_caller::<Env>(accounts.bob);
            let withdrawal_amount = 500;
            let withdrawal_id = contract.create_withdrawal_request(withdrawal_amount).expect("Should create withdrawal request");
            
            // Verify the request ID is 2
            assert_eq!(withdrawal_id, 2);
            
            // Verify the request count is updated
            assert_eq!(contract.get_request_count(RequestType::Withdrawal), 1);
            
            // Verify the request exists and has the correct data
            let request = contract.get_request(withdrawal_id).expect("Request should exist");
            assert_eq!(request.id, withdrawal_id);
            assert_eq!(request.request_type, RequestType::Withdrawal);
            assert_eq!(request.wallet_address, accounts.bob);
            assert_eq!(request.amount, withdrawal_amount);
            assert_eq!(request.collateral_amount, None);
            assert!(!request.is_processed);
            
            // Verify the user's balances are updated
            let user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(user.active_balance, deposit_amount - withdrawal_amount);
            assert_eq!(user.pending_withdrawals, withdrawal_amount);
            
            // Process the withdrawal as admin
            test::set_caller::<Env>(accounts.alice); // Admin/owner
            contract.process_withdrawal_request(withdrawal_id).expect("Should process withdrawal");
            
            // Verify the request is now processed
            let processed_request = contract.get_request(withdrawal_id).expect("Request should exist");
            assert!(processed_request.is_processed);
            
            // Verify the user's balances are updated
            let updated_user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(updated_user.active_balance, deposit_amount - withdrawal_amount);
            assert_eq!(updated_user.pending_withdrawals, 0);
        }
        
        /// Test withdrawal request with insufficient funds
        #[ink::test]
        fn test_withdrawal_insufficient_funds() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for this test
            test::set_caller::<Env>(accounts.bob);
            
            // First create a deposit to have some funds
            let deposit_amount = 500;
            let deposit_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Process the deposit as admin to make funds available
            test::set_caller::<Env>(accounts.alice); // Admin/owner
            contract.process_deposit_request(deposit_id).expect("Should process deposit");
            
            // Now try to withdraw more than available
            test::set_caller::<Env>(accounts.bob);
            let withdrawal_amount = 1000; // More than deposited
            let result = contract.create_withdrawal_request(withdrawal_amount);
            
            // Verify the request fails with the expected error
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Insufficient balance for withdrawal");
            
            // Verify no withdrawal request was created
            assert_eq!(contract.get_request_count(RequestType::Withdrawal), 0);
            
            // Verify the user's balances are unchanged
            let user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(user.active_balance, deposit_amount);
            assert_eq!(user.pending_withdrawals, 0);
        }
        
        /// Test validation for minimum deposit amount
        #[ink::test]
        fn test_minimum_deposit_amount() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for this test
            test::set_caller::<Env>(accounts.bob);
            
            // Try to create a deposit request with amount below minimum
            let deposit_amount = 5; // Below the minimum of 10
            let result = contract.create_deposit_request(deposit_amount);
            
            // Verify the request fails with the expected error
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Amount is below minimum deposit amount");
            
            // Verify no request was created
            assert_eq!(contract.get_request_count(RequestType::Deposit), 0);
        }
        
        /// Test validation for minimum withdrawal amount
        #[ink::test]
        fn test_minimum_withdrawal_amount() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // First create a deposit to have funds
            test::set_caller::<Env>(accounts.bob);
            let deposit_amount = 1000;
            let deposit_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Process the deposit as admin
            test::set_caller::<Env>(accounts.alice); // Admin/owner
            contract.process_deposit_request(deposit_id).expect("Should process deposit");
            
            // Try to create a withdrawal request with amount below minimum
            test::set_caller::<Env>(accounts.bob);
            let withdrawal_amount = 5; // Below the minimum of 10
            let result = contract.create_withdrawal_request(withdrawal_amount);
            
            // Verify the request fails with the expected error
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Amount is below minimum withdrawal amount");
            
            // Verify no withdrawal request was created
            assert_eq!(contract.get_request_count(RequestType::Withdrawal), 0);
        }
        
        /// Test validation for maximum pending requests
        #[ink::test]
        fn test_max_pending_requests() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for this test
            test::set_caller::<Env>(accounts.bob);
            
            // Create max number of deposit requests (5)
            for i in 0..5 {
                let deposit_amount = 100 * (i as u128 + 1);
                let request_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
                assert_eq!(request_id, i as u128 + 1);
            }
            
            // Try to create one more deposit request
            let deposit_amount = 600;
            let result = contract.create_deposit_request(deposit_amount);
            
            // Verify the request fails with the expected error
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Too many pending requests");
            
            // Verify only 5 requests were created
            assert_eq!(contract.get_request_count(RequestType::Deposit), 5);
        }
        
        /// Test validation for daily withdrawal limit
        #[ink::test]
        fn test_daily_withdrawal_limit() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // First create a deposit to have funds
            test::set_caller::<Env>(accounts.bob);
            let deposit_amount = 20000; // More than the daily withdrawal limit
            let deposit_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Process the deposit as admin
            test::set_caller::<Env>(accounts.alice); // Admin/owner
            contract.process_deposit_request(deposit_id).expect("Should process deposit");
            
            // Create a withdrawal request at the daily limit
            test::set_caller::<Env>(accounts.bob);
            let withdrawal_amount = 10000; // Equal to the daily limit
            let request_id = contract.create_withdrawal_request(withdrawal_amount).expect("Should create withdrawal request");
            
            // Verify the request was created
            assert_eq!(request_id, 2);
            
            // Try to create another withdrawal request
            let second_withdrawal_amount = 1000;
            let result = contract.create_withdrawal_request(second_withdrawal_amount);
            
            // Verify the request fails with the expected error
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Daily withdrawal limit exceeded");
            
            // Verify only 1 withdrawal request was created
            assert_eq!(contract.get_request_count(RequestType::Withdrawal), 1);
        }
        
        /// Test updating validation parameters
        #[ink::test]
        fn test_update_validation_parameters() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Alice (owner) for this test
            test::set_caller::<Env>(accounts.alice);
            
            // Update the daily withdrawal limit
            contract.set_daily_withdrawal_limit(20000).expect("Should update daily withdrawal limit");
            
            // Update the minimum deposit amount
            contract.set_min_deposit_amount(20).expect("Should update minimum deposit amount");
            
            // Update the minimum withdrawal amount
            contract.set_min_withdrawal_amount(50).expect("Should update minimum withdrawal amount");
            
            // Update the maximum pending requests
            contract.set_max_pending_requests(10).expect("Should update maximum pending requests");
            
            // Try to create a deposit with the old minimum (should fail)
            test::set_caller::<Env>(accounts.bob);
            let deposit_amount = 15; // Between old (10) and new (20) minimum
            let result = contract.create_deposit_request(deposit_amount);
            
            // Verify the request fails with the expected error
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Amount is below minimum deposit amount");
            
            // Try with the new minimum (should succeed)
            let deposit_amount = 20;
            let request_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            assert_eq!(request_id, 1);
        }
    }
} 