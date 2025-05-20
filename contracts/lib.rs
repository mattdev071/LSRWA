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
        is_kyc_approved: bool,       // Whether the user's KYC is approved
        kyc_timestamp: Option<Timestamp>, // KYC approval timestamp
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

    /// Event emitted when KYC status changes
    #[ink(event)]
    pub struct KycStatusChanged {
        #[ink(topic)]
        wallet_address: AccountId,
        is_approved: bool,
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
    }
} 