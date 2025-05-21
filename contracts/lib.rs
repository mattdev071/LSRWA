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
        
        /// Creates a new deposit request
        /// 
        /// # Arguments
        /// * `amount` - The amount to deposit
        /// 
        /// # Returns
        /// The ID of the created request
        #[ink(message, payable)]
        pub fn create_deposit_request(&mut self, amount: Balance) -> Result<u128, &'static str> {
            // Get the caller's wallet address
            let caller = Self::env().caller();
            
            // Ensure amount is greater than zero
            if amount == 0 {
                return Err("Amount must be greater than zero");
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
        
        /// Checks if a user is registered
        /// 
        /// # Arguments
        /// * `wallet_address` - The wallet address to check
        /// 
        /// # Returns
        /// True if the user is registered, false otherwise
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
        /// 
        /// # Arguments
        /// * `wallet_address` - The wallet address to update
        /// * `is_approved` - Whether the registration is approved
        /// 
        /// # Returns
        /// Result indicating success or failure
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
        /// 
        /// # Arguments
        /// * `request_id` - The ID of the request to process
        /// 
        /// # Returns
        /// Result indicating success or failure
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
    }
} 