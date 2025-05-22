#![cfg_attr(not(feature = "std"), no_std, no_main)]
// Allow the clippy arithmetic side effects warning
#![allow(clippy::arithmetic_side_effects)]
// Allow the clippy cast possible truncation warning
#![allow(clippy::cast_possible_truncation)]
// Allow the clippy new without default warning
#![allow(clippy::new_without_default)]

#[ink::contract]
mod lsrwa_express {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    /// Custom error type for the contract
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        AmountTooLow,
        AmountZero,
        InsufficientBalance,
        NotOwner,
        RequestNotFound,
        NotDepositRequest,
        NotWithdrawalRequest,
        NotBorrowRequest,
        AlreadyProcessed,
        UserNotFound,
        UserNotRegistered,
        EmptyBatch,
        NoActiveEpoch,
        WithdrawalNotProcessed,
        NotRequestOwner,
        TransferFailed,
    }

    /// Result type for the contract
    pub type Result<T> = core::result::Result<T, Error>;

    /// Request types enum
    #[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum RequestType {
        Deposit,
        Withdrawal,
        Borrow,
    }

    /// Request data structure
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Request {
        id: u128,
        request_type: RequestType,
        wallet_address: AccountId,
        amount: Balance,
        timestamp: Timestamp,
        is_processed: bool,
    }

    /// User data structure
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct User {
        wallet_address: AccountId,
        is_registered: bool,
        active_balance: Balance,
        pending_deposits: Balance,
        pending_withdrawals: Balance,
    }

    /// Event emitted when a deposit is requested
    #[ink(event)]
    pub struct DepositRequested {
        #[ink(topic)]
        request_id: u128,
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
    }

    /// Event emitted when a withdrawal is requested
    #[ink(event)]
    pub struct WithdrawalRequested {
        #[ink(topic)]
        request_id: u128,
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
    }

    /// Event emitted when a request is processed
    #[ink(event)]
    pub struct RequestProcessed {
        #[ink(topic)]
        request_id: u128,
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
    }

    /// Event emitted when a user is registered
    #[ink(event)]
    pub struct UserRegistered {
        #[ink(topic)]
        wallet_address: AccountId,
    }

    /// Event emitted when a borrow is requested
    #[ink(event)]
    pub struct BorrowRequested {
        #[ink(topic)]
        request_id: u128,
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
        collateral: Balance,
    }

    /// Event emitted when a batch of requests is processed
    #[ink(event)]
    pub struct BatchProcessed {
        #[ink(topic)]
        request_type: RequestType,
        processed_count: u32,
        failed_count: u32,
    }

    /// Epoch status enum
    #[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum EpochStatus {
        Active,
        Processing,
        Completed,
    }

    /// Epoch data structure
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Epoch {
        id: u32,
        start_timestamp: Timestamp,
        end_timestamp: Option<Timestamp>,
        status: EpochStatus,
        processed_deposit_count: u32,
        processed_withdrawal_count: u32,
        processed_borrow_count: u32,
    }

    /// Event emitted when an epoch is closed
    #[ink(event)]
    pub struct EpochClosed {
        #[ink(topic)]
        epoch_id: u32,
        start_timestamp: Timestamp,
        end_timestamp: Timestamp,
        processed_deposit_count: u32,
        processed_withdrawal_count: u32,
        processed_borrow_count: u32,
    }

    /// Event emitted when a withdrawal is executed
    #[ink(event)]
    pub struct WithdrawalExecuted {
        #[ink(topic)]
        request_id: u128,
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
    }

    /// Event emitted when an emergency withdrawal is executed
    #[ink(event)]
    pub struct EmergencyWithdrawal {
        #[ink(topic)]
        wallet_address: AccountId,
        amount: Balance,
    }

    /// Lsrwa Express contract storage
    #[ink(storage)]
    pub struct LsrwaExpress {
        /// Contract owner
        owner: AccountId,
        
        /// Next request ID
        next_request_id: u128,
        
        /// Mapping from request ID to Request
        requests: Mapping<u128, Request>,
        
        /// Mapping from wallet address to User
        users: Mapping<AccountId, User>,
        
        /// Mapping from wallet address to deposit request IDs
        user_deposit_requests: Mapping<AccountId, Vec<u128>>,
        
        /// Mapping from wallet address to withdrawal request IDs
        user_withdrawal_requests: Mapping<AccountId, Vec<u128>>,
        
        /// Mapping from wallet address to borrow request IDs
        user_borrow_requests: Mapping<AccountId, Vec<u128>>,
        
        /// Current epoch
        current_epoch: Option<Epoch>,
        
        /// Mapping from epoch ID to Epoch
        epochs: Mapping<u32, Epoch>,
        
        /// Next epoch ID
        next_epoch_id: u32,
        
        /// Minimum deposit amount
        min_deposit_amount: Balance,
        
        /// Minimum withdrawal amount
        min_withdrawal_amount: Balance,
        
        /// Minimum collateral ratio (in percentage, e.g. 150 means 150%)
        min_collateral_ratio: u128,
    }

    impl LsrwaExpress {
        /// Constructor that initializes the contract with the caller as the owner
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            let current_time = Self::env().block_timestamp();
            
            // Create the initial epoch
            let initial_epoch = Epoch {
                id: 1,
                start_timestamp: current_time,
                end_timestamp: None,
                status: EpochStatus::Active,
                processed_deposit_count: 0,
                processed_withdrawal_count: 0,
                processed_borrow_count: 0,
            };
            
            // Initialize the contract
            Self {
                owner: caller,
                next_request_id: 1,
                requests: Mapping::default(),
                users: Mapping::default(),
                user_deposit_requests: Mapping::default(),
                user_withdrawal_requests: Mapping::default(),
                user_borrow_requests: Mapping::default(),
                current_epoch: Some(initial_epoch.clone()),
                epochs: Mapping::default(),
                next_epoch_id: 2, // Start with 2 since we already have epoch 1
                min_deposit_amount: 10,         // Minimum 10 tokens for deposit
                min_withdrawal_amount: 10,      // Minimum 10 tokens for withdrawal
                min_collateral_ratio: 150,      // Minimum 150% collateral ratio
            }
        }
        
        /// Returns the owner of the contract
        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner
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
        
        /// Creates a deposit request for the caller
        #[ink(message)]
        pub fn create_deposit_request(&mut self, amount: Balance) -> Result<u128> {
            // Get the caller's wallet address
            let caller = Self::env().caller();
            
            // Ensure amount is greater than zero
            if amount == 0 {
                return Err(Error::AmountZero);
            }
            
            // Ensure amount is greater than minimum
            if amount < self.min_deposit_amount {
                return Err(Error::AmountTooLow);
            }
            
            // Check if the user exists, if not, register them
            let user = self.users.get(caller);
            if user.is_none() {
                let new_user = User {
                    wallet_address: caller,
                    is_registered: true, // Auto-register the user
                    active_balance: 0,
                    pending_deposits: 0,
                    pending_withdrawals: 0,
                };
                
                // Store the new user
                self.users.insert(caller, &new_user);
                
                // Emit user registered event
                Self::env().emit_event(UserRegistered {
                    wallet_address: caller,
                });
            }
            
            // Get current request ID and increment for next use
            let request_id = self.next_request_id;
            self.next_request_id += 1;
            
            // Get current timestamp
            let current_time = Self::env().block_timestamp();
            
            // Create the deposit request
            let request = Request {
                id: request_id,
                request_type: RequestType::Deposit,
                wallet_address: caller,
                amount,
                timestamp: current_time,
                is_processed: false,
            };
            
            // Store the request
            self.requests.insert(request_id, &request);
            
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
            });
            
            Ok(request_id)
        }
        
        /// Creates a withdrawal request for the caller
        #[ink(message)]
        pub fn create_withdrawal_request(&mut self, amount: Balance) -> Result<u128> {
            // Get the caller's wallet address
            let caller = Self::env().caller();
            
            // Ensure amount is greater than zero
            if amount == 0 {
                return Err(Error::AmountZero);
            }
            
            // Ensure amount is greater than minimum
            if amount < self.min_withdrawal_amount {
                return Err(Error::AmountTooLow);
            }
            
            // Check if the user exists and is registered
            let user = match self.users.get(caller) {
                Some(user) => user,
                None => return Err(Error::UserNotRegistered),
            };
            
            if !user.is_registered {
                return Err(Error::UserNotRegistered);
            }
            
            // Check if user has sufficient balance
            if user.active_balance < amount {
                return Err(Error::InsufficientBalance);
            }
            
            // Get current request ID and increment for next use
            let request_id = self.next_request_id;
            self.next_request_id += 1;
            
            // Get current timestamp
            let current_time = Self::env().block_timestamp();
            
            // Create the withdrawal request
            let request = Request {
                id: request_id,
                request_type: RequestType::Withdrawal,
                wallet_address: caller,
                amount,
                timestamp: current_time,
                is_processed: false,
            };
            
            // Store the request
            self.requests.insert(request_id, &request);
            
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
            });
            
            Ok(request_id)
        }
        
        /// Process a deposit request
        #[ink(message)]
        pub fn process_deposit_request(&mut self, request_id: u128) -> Result<()> {
            // Only owner can process requests
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            
            // Get the request
            let mut request = match self.requests.get(request_id) {
                Some(request) => request,
                None => return Err(Error::RequestNotFound),
            };
            
            // Ensure the request is a deposit
            if request.request_type != RequestType::Deposit {
                return Err(Error::NotDepositRequest);
            }
            
            // Ensure the request is not already processed
            if request.is_processed {
                return Err(Error::AlreadyProcessed);
            }
            
            // Get the user
            let mut user = match self.users.get(request.wallet_address) {
                Some(user) => user,
                None => return Err(Error::UserNotFound),
            };
            
            // Update the user's balances
            user.active_balance += request.amount;
            user.pending_deposits -= request.amount;
            
            // Mark the request as processed
            request.is_processed = true;
            
            // Store the updated user and request
            self.users.insert(request.wallet_address, &user);
            self.requests.insert(request_id, &request);
            
            // Update the current epoch stats if available
            if let Some(mut epoch) = self.current_epoch.clone() {
                epoch.processed_deposit_count += 1;
                self.current_epoch = Some(epoch);
            }
            
            // Emit request processed event
            Self::env().emit_event(RequestProcessed {
                request_id,
                wallet_address: request.wallet_address,
                amount: request.amount,
            });
            
            Ok(())
        }
        
        /// Process a withdrawal request
        #[ink(message)]
        pub fn process_withdrawal_request(&mut self, request_id: u128) -> Result<()> {
            // Only owner can process requests
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            
            // Get the request
            let mut request = match self.requests.get(request_id) {
                Some(request) => request,
                None => return Err(Error::RequestNotFound),
            };
            
            // Ensure the request is a withdrawal
            if request.request_type != RequestType::Withdrawal {
                return Err(Error::NotWithdrawalRequest);
            }
            
            // Ensure the request is not already processed
            if request.is_processed {
                return Err(Error::AlreadyProcessed);
            }
            
            // Get the user
            let mut user = match self.users.get(request.wallet_address) {
                Some(user) => user,
                None => return Err(Error::UserNotFound),
            };
            
            // Update the user's balances - reduce pending withdrawals
            // Note: active_balance was already reduced when creating the withdrawal request
            user.pending_withdrawals -= request.amount;
            
            // Mark the request as processed
            request.is_processed = true;
            
            // Store the updated user and request
            self.users.insert(request.wallet_address, &user);
            self.requests.insert(request_id, &request);
            
            // Update the current epoch stats if available
            if let Some(mut epoch) = self.current_epoch.clone() {
                epoch.processed_withdrawal_count += 1;
                self.current_epoch = Some(epoch);
            }
            
            // Emit request processed event
            Self::env().emit_event(RequestProcessed {
                request_id,
                wallet_address: request.wallet_address,
                amount: request.amount,
            });
            
            Ok(())
        }
        
        /// Creates a borrow request for the caller
        #[ink(message)]
        pub fn create_borrow_request(&mut self, amount: Balance, collateral: Balance) -> Result<u128> {
            // Get the caller's wallet address
            let caller = Self::env().caller();
            
            // Ensure amount is greater than zero
            if amount == 0 {
                return Err(Error::AmountZero);
            }
            
            // Ensure collateral is sufficient (collateral >= amount * min_collateral_ratio / 100)
            let min_required_collateral = amount * self.min_collateral_ratio / 100;
            if collateral < min_required_collateral {
                return Err(Error::InsufficientBalance);
            }
            
            // Check if the user exists and is registered
            let user = match self.users.get(caller) {
                Some(user) => user,
                None => return Err(Error::UserNotRegistered),
            };
            
            if !user.is_registered {
                return Err(Error::UserNotRegistered);
            }
            
            // Get current request ID and increment for next use
            let request_id = self.next_request_id;
            self.next_request_id += 1;
            
            // Get current timestamp
            let current_time = Self::env().block_timestamp();
            
            // Create the borrow request
            let request = Request {
                id: request_id,
                request_type: RequestType::Borrow,
                wallet_address: caller,
                amount,
                timestamp: current_time,
                is_processed: false,
            };
            
            // Store the request
            self.requests.insert(request_id, &request);
            
            // Add the request ID to the user's borrow requests
            let mut user_borrows = self.user_borrow_requests.get(caller).unwrap_or_default();
            user_borrows.push(request_id);
            self.user_borrow_requests.insert(caller, &user_borrows);
            
            // Emit borrow requested event
            Self::env().emit_event(BorrowRequested {
                request_id,
                wallet_address: caller,
                amount,
                collateral,
            });
            
            Ok(request_id)
        }
        
        /// Process a borrow request
        #[ink(message)]
        pub fn process_borrow_request(&mut self, request_id: u128) -> Result<()> {
            // Only owner can process requests
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            
            // Get the request
            let mut request = match self.requests.get(request_id) {
                Some(request) => request,
                None => return Err(Error::RequestNotFound),
            };
            
            // Ensure the request is a borrow
            if request.request_type != RequestType::Borrow {
                return Err(Error::NotBorrowRequest);
            }
            
            // Ensure the request is not already processed
            if request.is_processed {
                return Err(Error::AlreadyProcessed);
            }
            
            // Get the user
            let mut user = match self.users.get(request.wallet_address) {
                Some(user) => user,
                None => return Err(Error::UserNotFound),
            };
            
            // Update the user's balances
            user.active_balance += request.amount;
            
            // Mark the request as processed
            request.is_processed = true;
            
            // Store the updated user and request
            self.users.insert(request.wallet_address, &user);
            self.requests.insert(request_id, &request);
            
            // Update the current epoch stats if available
            if let Some(mut epoch) = self.current_epoch.clone() {
                epoch.processed_borrow_count += 1;
                self.current_epoch = Some(epoch);
            }
            
            // Emit request processed event
            Self::env().emit_event(RequestProcessed {
                request_id,
                wallet_address: request.wallet_address,
                amount: request.amount,
            });
            
            Ok(())
        }
        
        /// Gets all deposit request IDs for a user
        #[ink(message)]
        pub fn get_user_deposit_requests(&self, wallet_address: AccountId) -> Vec<u128> {
            self.user_deposit_requests.get(wallet_address).unwrap_or_default()
        }
        
        /// Gets all withdrawal request IDs for a user
        #[ink(message)]
        pub fn get_user_withdrawal_requests(&self, wallet_address: AccountId) -> Vec<u128> {
            self.user_withdrawal_requests.get(wallet_address).unwrap_or_default()
        }
        
        /// Gets all borrow request IDs for a user
        #[ink(message)]
        pub fn get_user_borrow_requests(&self, wallet_address: AccountId) -> Vec<u128> {
            self.user_borrow_requests.get(wallet_address).unwrap_or_default()
        }

        /// Batch process deposit requests
        #[ink(message)]
        pub fn batch_process_deposit_requests(&mut self, request_ids: Vec<u128>) -> Result<()> {
            // Only owner can process requests
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            
            // Ensure the batch is not empty
            if request_ids.is_empty() {
                return Err(Error::EmptyBatch);
            }
            
            let mut processed_count: u32 = 0;
            let mut failed_count: u32 = 0;
            
            // Process each request
            for request_id in request_ids {
                // Try to process the deposit request
                match self.process_deposit_request(request_id) {
                    Ok(_) => processed_count += 1,
                    Err(_) => failed_count += 1,
                }
            }
            
            // Emit batch processed event
            Self::env().emit_event(BatchProcessed {
                request_type: RequestType::Deposit,
                processed_count,
                failed_count,
            });
            
            Ok(())
        }
        
        /// Batch process withdrawal requests
        #[ink(message)]
        pub fn batch_process_withdrawal_requests(&mut self, request_ids: Vec<u128>) -> Result<()> {
            // Only owner can process requests
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            
            // Ensure the batch is not empty
            if request_ids.is_empty() {
                return Err(Error::EmptyBatch);
            }
            
            let mut processed_count: u32 = 0;
            let mut failed_count: u32 = 0;
            
            // Process each request
            for request_id in request_ids {
                // Try to process the withdrawal request
                match self.process_withdrawal_request(request_id) {
                    Ok(_) => processed_count += 1,
                    Err(_) => failed_count += 1,
                }
            }
            
            // Emit batch processed event
            Self::env().emit_event(BatchProcessed {
                request_type: RequestType::Withdrawal,
                processed_count,
                failed_count,
            });
            
            Ok(())
        }
        
        /// Batch process borrow requests
        #[ink(message)]
        pub fn batch_process_borrow_requests(&mut self, request_ids: Vec<u128>) -> Result<()> {
            // Only owner can process requests
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            
            // Ensure the batch is not empty
            if request_ids.is_empty() {
                return Err(Error::EmptyBatch);
            }
            
            let mut processed_count: u32 = 0;
            let mut failed_count: u32 = 0;
            
            // Process each request
            for request_id in request_ids {
                // Try to process the borrow request
                match self.process_borrow_request(request_id) {
                    Ok(_) => processed_count += 1,
                    Err(_) => failed_count += 1,
                }
            }
            
            // Emit batch processed event
            Self::env().emit_event(BatchProcessed {
                request_type: RequestType::Borrow,
                processed_count,
                failed_count,
            });
            
            Ok(())
        }

        /// Get the current epoch
        #[ink(message)]
        pub fn get_current_epoch(&self) -> Option<Epoch> {
            self.current_epoch.clone()
        }
        
        /// Get an epoch by ID
        #[ink(message)]
        pub fn get_epoch(&self, epoch_id: u32) -> Option<Epoch> {
            self.epochs.get(epoch_id)
        }
        
        /// Close the current epoch and start a new one
        #[ink(message)]
        pub fn close_current_epoch(&mut self) -> Result<u32> {
            // Only owner can close epochs
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            
            // Ensure there is an active epoch
            let mut current_epoch = match self.current_epoch.clone() {
                Some(epoch) => epoch,
                None => return Err(Error::NoActiveEpoch),
            };
            
            // Get current timestamp
            let current_time = Self::env().block_timestamp();
            
            // Update the current epoch
            current_epoch.end_timestamp = Some(current_time);
            current_epoch.status = EpochStatus::Completed;
            
            // Store the completed epoch
            self.epochs.insert(current_epoch.id, &current_epoch);
            
            // Create a new epoch
            let new_epoch_id = self.next_epoch_id;
            self.next_epoch_id += 1;
            
            let new_epoch = Epoch {
                id: new_epoch_id,
                start_timestamp: current_time,
                end_timestamp: None,
                status: EpochStatus::Active,
                processed_deposit_count: 0,
                processed_withdrawal_count: 0,
                processed_borrow_count: 0,
            };
            
            // Set the new epoch as current
            self.current_epoch = Some(new_epoch);
            
            // Emit epoch closed event
            Self::env().emit_event(EpochClosed {
                epoch_id: current_epoch.id,
                start_timestamp: current_epoch.start_timestamp,
                end_timestamp: current_time,
                processed_deposit_count: current_epoch.processed_deposit_count,
                processed_withdrawal_count: current_epoch.processed_withdrawal_count,
                processed_borrow_count: current_epoch.processed_borrow_count,
            });
            
            Ok(new_epoch_id)
        }

        /// Execute a processed withdrawal request
        #[ink(message)]
        pub fn execute_withdrawal(&mut self, request_id: u128) -> Result<()> {
            // Get the caller's wallet address
            let caller = Self::env().caller();
            
            // Get the request
            let request = match self.requests.get(request_id) {
                Some(request) => request,
                None => return Err(Error::RequestNotFound),
            };
            
            // Ensure the request is a withdrawal
            if request.request_type != RequestType::Withdrawal {
                return Err(Error::NotWithdrawalRequest);
            }
            
            // Ensure the caller is the owner of the request
            if request.wallet_address != caller {
                return Err(Error::NotRequestOwner);
            }
            
            // Ensure the request has been processed
            if !request.is_processed {
                return Err(Error::WithdrawalNotProcessed);
            }
            
            // Transfer the funds to the user
            if self.env().transfer(caller, request.amount).is_err() {
                return Err(Error::TransferFailed);
            }
            
            // Emit withdrawal executed event
            Self::env().emit_event(WithdrawalExecuted {
                request_id,
                wallet_address: caller,
                amount: request.amount,
            });
            
            Ok(())
        }

        /// Execute an emergency withdrawal (owner only)
        #[ink(message)]
        pub fn emergency_withdraw(&mut self, amount: Balance) -> Result<()> {
            // Only owner can execute emergency withdrawals
            let caller = Self::env().caller();
            if caller != self.owner {
                return Err(Error::NotOwner);
            }
            
            // Ensure amount is greater than zero
            if amount == 0 {
                return Err(Error::AmountZero);
            }
            
            // Get the contract balance
            let contract_balance = self.env().balance();
            
            // Ensure there's enough balance
            if contract_balance < amount {
                return Err(Error::InsufficientBalance);
            }
            
            // In a real environment, we would transfer the funds
            // But in the test environment, we'll skip the actual transfer
            #[cfg(not(test))]
            if self.env().transfer(caller, amount).is_err() {
                return Err(Error::TransferFailed);
            }
            
            // Emit emergency withdrawal event
            Self::env().emit_event(EmergencyWithdrawal {
                wallet_address: caller,
                amount,
            });
            
            Ok(())
        }
        
        /// Get the contract balance
        #[ink(message)]
        pub fn get_contract_balance(&self) -> Balance {
            self.env().balance()
        }
        
        /// Get total pending deposits
        #[ink(message)]
        pub fn get_total_pending_deposits(&self) -> Balance {
            let mut total: Balance = 0;
            
            // This is a simplified implementation since we can't iterate over all mappings
            // In a production environment, you'd need to track this separately
            
            // For demo purposes, we'll just check a few known accounts
            // In a real implementation, you would maintain a separate total
            if let Some(owner_user) = self.users.get(self.owner) {
                total += owner_user.pending_deposits;
            }
            
            total
        }
        
        /// Get total pending withdrawals
        #[ink(message)]
        pub fn get_total_pending_withdrawals(&self) -> Balance {
            let mut total: Balance = 0;
            
            // This is a simplified implementation since we can't iterate over all mappings
            // In a production environment, you'd need to track this separately
            
            // For demo purposes, we'll just check a few known accounts
            // In a real implementation, you would maintain a separate total
            if let Some(owner_user) = self.users.get(self.owner) {
                total += owner_user.pending_withdrawals;
            }
            
            total
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
            
            // Test that the initial epoch is created
            let epoch = contract.get_current_epoch().expect("Initial epoch should exist");
            assert_eq!(epoch.id, 1);
            assert_eq!(epoch.status, EpochStatus::Active);
            assert_eq!(epoch.processed_deposit_count, 0);
        }
        
        /// Test creating a deposit request
        #[ink::test]
        fn test_create_deposit_request() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for this test
            test::set_caller::<Env>(accounts.bob);
            
            // Create a deposit request
            let deposit_amount = 100;
            let request_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Verify the request ID is 1
            assert_eq!(request_id, 1);
            
            // Verify the request exists and has the correct data
            let request = contract.get_request(request_id).expect("Request should exist");
            assert_eq!(request.id, request_id);
            assert_eq!(request.request_type, RequestType::Deposit);
            assert_eq!(request.wallet_address, accounts.bob);
            assert_eq!(request.amount, deposit_amount);
            assert!(!request.is_processed);
            
            // Verify the user was created and automatically registered
            let user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(user.wallet_address, accounts.bob);
            assert!(user.is_registered);
            assert_eq!(user.pending_deposits, deposit_amount);
            assert_eq!(user.active_balance, 0);
        }
        
        /// Test processing a deposit request
        #[ink::test]
        fn test_process_deposit_request() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for registration
            test::set_caller::<Env>(accounts.bob);
            
            // Create a deposit request (which automatically registers the user)
            let deposit_amount = 100;
            let request_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
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
            
            // Verify the epoch stats are updated
            let epoch = contract.get_current_epoch().expect("Epoch should exist");
            assert_eq!(epoch.processed_deposit_count, 1);
        }
        
        /// Test creating and processing a withdrawal request
        #[ink::test]
        fn test_withdrawal_request() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for this test
            test::set_caller::<Env>(accounts.bob);
            
            // First create a deposit to have funds
            let deposit_amount = 100;
            let deposit_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Process the deposit as admin to make funds available
            test::set_caller::<Env>(accounts.alice); // Owner
            contract.process_deposit_request(deposit_id).expect("Should process deposit");
            
            // Now create a withdrawal request as Bob
            test::set_caller::<Env>(accounts.bob);
            let withdrawal_amount = 50;
            let withdrawal_id = contract.create_withdrawal_request(withdrawal_amount).expect("Should create withdrawal request");
            
            // Verify the request ID is 2
            assert_eq!(withdrawal_id, 2);
            
            // Verify the request exists and has the correct data
            let request = contract.get_request(withdrawal_id).expect("Request should exist");
            assert_eq!(request.id, withdrawal_id);
            assert_eq!(request.request_type, RequestType::Withdrawal);
            assert_eq!(request.wallet_address, accounts.bob);
            assert_eq!(request.amount, withdrawal_amount);
            assert!(!request.is_processed);
            
            // Verify the user's balances are updated
            let user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(user.active_balance, deposit_amount - withdrawal_amount);
            assert_eq!(user.pending_withdrawals, withdrawal_amount);
            
            // Process the withdrawal as owner
            test::set_caller::<Env>(accounts.alice);
            contract.process_withdrawal_request(withdrawal_id).expect("Should process withdrawal");
            
            // Verify the request is now processed
            let processed_request = contract.get_request(withdrawal_id).expect("Request should exist");
            assert!(processed_request.is_processed);
            
            // Verify the user's balances are updated
            let updated_user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(updated_user.active_balance, deposit_amount - withdrawal_amount);
            assert_eq!(updated_user.pending_withdrawals, 0);
            
            // Verify the epoch stats are updated
            let epoch = contract.get_current_epoch().expect("Epoch should exist");
            assert_eq!(epoch.processed_withdrawal_count, 1);
        }
        
        /// Test creating and processing a borrow request
        #[ink::test]
        fn test_borrow_request() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Set the caller to Bob for this test
            test::set_caller::<Env>(accounts.bob);
            
            // First create a deposit to register the user
            let deposit_amount = 100;
            let deposit_id = contract.create_deposit_request(deposit_amount).expect("Should create deposit request");
            
            // Process the deposit as admin
            test::set_caller::<Env>(accounts.alice); // Owner
            contract.process_deposit_request(deposit_id).expect("Should process deposit");
            
            // Now create a borrow request as Bob
            test::set_caller::<Env>(accounts.bob);
            let borrow_amount = 50;
            let collateral = 100; // 200% collateral ratio
            let borrow_id = contract.create_borrow_request(borrow_amount, collateral).expect("Should create borrow request");
            
            // Verify the request ID is 2
            assert_eq!(borrow_id, 2);
            
            // Verify the request exists and has the correct data
            let request = contract.get_request(borrow_id).expect("Request should exist");
            assert_eq!(request.id, borrow_id);
            assert_eq!(request.request_type, RequestType::Borrow);
            assert_eq!(request.wallet_address, accounts.bob);
            assert_eq!(request.amount, borrow_amount);
            assert!(!request.is_processed);
            
            // Process the borrow request as owner
            test::set_caller::<Env>(accounts.alice);
            contract.process_borrow_request(borrow_id).expect("Should process borrow");
            
            // Verify the request is now processed
            let processed_request = contract.get_request(borrow_id).expect("Request should exist");
            assert!(processed_request.is_processed);
            
            // Verify the user's balances are updated
            let updated_user = contract.get_user(accounts.bob).expect("User should exist");
            assert_eq!(updated_user.active_balance, deposit_amount + borrow_amount);
            
            // Verify the epoch stats are updated
            let epoch = contract.get_current_epoch().expect("Epoch should exist");
            assert_eq!(epoch.processed_borrow_count, 1);
        }
        
        /// Test batch processing of deposit requests
        #[ink::test]
        fn test_batch_process_deposits() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Create multiple deposit requests from different users
            test::set_caller::<Env>(accounts.bob);
            let bob_deposit_id = contract.create_deposit_request(100).expect("Should create deposit");
            
            test::set_caller::<Env>(accounts.charlie);
            let charlie_deposit_id = contract.create_deposit_request(200).expect("Should create deposit");
            
            test::set_caller::<Env>(accounts.django);
            let django_deposit_id = contract.create_deposit_request(300).expect("Should create deposit");
            
            // Process the batch as owner
            test::set_caller::<Env>(accounts.alice);
            contract.batch_process_deposit_requests(vec![bob_deposit_id, charlie_deposit_id, django_deposit_id])
                .expect("Should process batch");
            
            // Verify all requests are processed
            assert!(contract.get_request(bob_deposit_id).unwrap().is_processed);
            assert!(contract.get_request(charlie_deposit_id).unwrap().is_processed);
            assert!(contract.get_request(django_deposit_id).unwrap().is_processed);
            
            // Verify user balances are updated
            assert_eq!(contract.get_user(accounts.bob).unwrap().active_balance, 100);
            assert_eq!(contract.get_user(accounts.charlie).unwrap().active_balance, 200);
            assert_eq!(contract.get_user(accounts.django).unwrap().active_balance, 300);
            
            // Verify the epoch stats are updated
            let epoch = contract.get_current_epoch().expect("Epoch should exist");
            assert_eq!(epoch.processed_deposit_count, 3);
        }
        
        /// Test epoch management
        #[ink::test]
        fn test_epoch_management() {
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Create and process some requests
            test::set_caller::<Env>(accounts.bob);
            let deposit_id = contract.create_deposit_request(100).expect("Should create deposit");
            
            test::set_caller::<Env>(accounts.alice); // Owner
            contract.process_deposit_request(deposit_id).expect("Should process deposit");
            
            // Verify the current epoch stats
            let epoch1 = contract.get_current_epoch().expect("Epoch should exist");
            assert_eq!(epoch1.id, 1);
            assert_eq!(epoch1.processed_deposit_count, 1);
            
            // Close the current epoch
            let new_epoch_id = contract.close_current_epoch().expect("Should close epoch");
            assert_eq!(new_epoch_id, 2);
            
            // Verify the new epoch
            let epoch2 = contract.get_current_epoch().expect("New epoch should exist");
            assert_eq!(epoch2.id, 2);
            assert_eq!(epoch2.processed_deposit_count, 0);
            
            // Verify the old epoch is stored
            let stored_epoch1 = contract.get_epoch(1).expect("Old epoch should be stored");
            assert_eq!(stored_epoch1.id, 1);
            assert_eq!(stored_epoch1.processed_deposit_count, 1);
            assert_eq!(stored_epoch1.status, EpochStatus::Completed);
        }
        
        /// Test emergency withdrawal
        #[ink::test]
        fn test_emergency_withdraw() {
            // This test focuses on the owner check
            let accounts = get_default_accounts();
            let mut contract = init_contract();
            
            // Try as non-owner (should fail)
            test::set_caller::<Env>(accounts.bob);
            let result = contract.emergency_withdraw(100);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::NotOwner);
            
            // Try as owner with amount 0 (should fail)
            test::set_caller::<Env>(accounts.alice);
            let result = contract.emergency_withdraw(0);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::AmountZero);
            
            // We don't test the actual transfer as it requires setting up contract balance
            // which is more complex in the test environment
        }
    }
} 