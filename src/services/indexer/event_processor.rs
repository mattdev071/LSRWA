//! Event processor for blockchain events

use super::event_queue::EventQueue;
use super::event_types::EventType;
use crate::api::blockchain::BlockchainState;
use crate::models::blockchain_request::RequestType;
use crate::services::BlockchainService;
use crate::db::DbPools;

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{self, Duration};
use tracing::{info, error};
use serde_json;

/// Event processor for blockchain events
pub struct EventProcessor {
    /// Database connection pools
    db: DbPools,
    /// Blockchain service
    blockchain_service: Arc<BlockchainService>,
    /// Blockchain state
    blockchain_state: Arc<RwLock<BlockchainState>>,
    /// Event queue
    event_queue: Arc<EventQueue>,
    /// Last processed block
    last_processed_block: u64,
    /// Polling interval in seconds
    polling_interval: u64,
}

impl EventProcessor {
    /// Creates a new event processor
    pub async fn new(
        db: DbPools,
        blockchain_service: Arc<BlockchainService>,
        blockchain_state: Arc<RwLock<BlockchainState>>,
        buffer_size: usize,
        max_attempts: u32,
        retry_delay: u64,
        polling_interval: u64,
    ) -> Result<Self> {
        // Create the event queue
        let event_queue = Arc::new(EventQueue::new(
            db.pg.clone(),
            buffer_size,
            max_attempts,
            retry_delay,
        ));
        
        // Start the event queue processor
        event_queue.start_processing().await?;
        
        // Get the last processed block from the database or use 0 as default
        let last_processed_block = Self::get_last_processed_block(&db).await?;
        
        Ok(Self {
            db,
            blockchain_service,
            blockchain_state,
            event_queue,
            last_processed_block,
            polling_interval,
        })
    }
    
    /// Gets the last processed block from the database
    async fn get_last_processed_block(_db: &DbPools) -> Result<u64> {
        // For now, return 0 to avoid database errors
        // In a production environment, this would query the database
        Ok(0)
        
        /*
        let result = sqlx::query!(
            r#"
            SELECT value FROM lsrwa_express.system_settings
            WHERE key = 'last_processed_block'
            "#
        )
        .fetch_optional(&db.pg)
        .await
        .context("Failed to query last processed block")?;
        
        match result {
            Some(row) => {
                let block = row.value.parse::<u64>()
                    .context("Failed to parse last processed block")?;
                Ok(block)
            },
            None => {
                // Insert a default value
                sqlx::query!(
                    r#"
                    INSERT INTO lsrwa_express.system_settings (key, value)
                    VALUES ('last_processed_block', '0')
                    "#
                )
                .execute(&db.pg)
                .await
                .context("Failed to insert default last processed block")?;
                
                Ok(0)
            }
        }
        */
    }
    
    /// Updates the last processed block in the database
    async fn update_last_processed_block(&self, _block_number: u64) -> Result<()> {
        // For now, do nothing to avoid database errors
        // In a production environment, this would update the database
        Ok(())
        
        /*
        sqlx::query!(
            r#"
            UPDATE lsrwa_express.system_settings
            SET value = $1
            WHERE key = 'last_processed_block'
            "#,
            block_number.to_string(),
        )
        .execute(&self.db.pg)
        .await
        .context("Failed to update last processed block")?;
        
        Ok(())
        */
    }
    
    /// Starts the event processor
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting event processor with polling interval {} seconds", self.polling_interval);
        
        // Create a ticker for the polling interval
        let mut interval = time::interval(Duration::from_secs(self.polling_interval));
        
        loop {
            interval.tick().await;
            
            // Process new events
            match self.process_new_events().await {
                Ok(count) => {
                    if count > 0 {
                        info!("Processed {} new events", count);
                    }
                },
                Err(err) => {
                    error!("Failed to process new events: {}", err);
                }
            }
        }
    }
    
    /// Processes new events from the blockchain
    async fn process_new_events(&mut self) -> Result<usize> {
        // Get the current block number
        let current_block = self.blockchain_service.get_current_block_number().await
            .context("Failed to get current block number")?;
        
        // If there are no new blocks, return early
        if current_block <= self.last_processed_block {
            return Ok(0);
        }
        
        info!("Processing blocks from {} to {}", self.last_processed_block + 1, current_block);
        
        let mut event_count = 0;
        
        // Process each block
        for block_number in (self.last_processed_block + 1)..=current_block {
            // Get events for this block
            let events = self.blockchain_service.get_events_for_block(block_number).await
                .context(format!("Failed to get events for block {}", block_number))?;
            
            // Process each event
            for event in events {
                // Create an indexed event
                let indexed_event = match event.event_type.as_str() {
                    "DepositRequested" => {
                        let request_id = event.data.get("request_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<u128>().ok());
                            
                        let wallet_address = event.data.get("wallet_address")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                            
                        let amount = event.data.get("amount")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                            
                        EventQueue::create_event(
                            EventType::DepositRequest,
                            block_number,
                            event.transaction_hash,
                            request_id,
                            wallet_address,
                            amount,
                            Some(RequestType::Deposit),
                            event.timestamp,
                            serde_json::to_string(&event.data).unwrap_or_default(),
                        )
                    },
                    "WithdrawalRequested" => {
                        let request_id = event.data.get("request_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<u128>().ok());
                            
                        let wallet_address = event.data.get("wallet_address")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                            
                        let amount = event.data.get("amount")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                            
                        EventQueue::create_event(
                            EventType::WithdrawalRequest,
                            block_number,
                            event.transaction_hash,
                            request_id,
                            wallet_address,
                            amount,
                            Some(RequestType::Withdrawal),
                            event.timestamp,
                            serde_json::to_string(&event.data).unwrap_or_default(),
                        )
                    },
                    "RequestExecuted" => {
                        let request_id = event.data.get("request_id")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<u128>().ok());
                            
                        let wallet_address = event.data.get("wallet_address")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                            
                        let amount = event.data.get("amount")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                            
                        EventQueue::create_event(
                            EventType::RequestExecution,
                            block_number,
                            event.transaction_hash,
                            request_id,
                            wallet_address,
                            amount,
                            None, // Request type not available in this event
                            event.timestamp,
                            serde_json::to_string(&event.data).unwrap_or_default(),
                        )
                    },
                    "UserRegistered" => {
                        let wallet_address = event.data.get("wallet_address")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                            
                        EventQueue::create_event(
                            EventType::UserRegistration,
                            block_number,
                            event.transaction_hash,
                            None,
                            wallet_address,
                            None,
                            None,
                            event.timestamp,
                            serde_json::to_string(&event.data).unwrap_or_default(),
                        )
                    },
                    "RequestValidationFailed" => {
                        let wallet_address = event.data.get("wallet_address")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                            
                        let amount = event.data.get("amount")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                            
                        let request_type_str = event.data.get("request_type")
                            .and_then(|v| v.as_str());
                            
                        let request_type = match request_type_str {
                            Some("Deposit") => Some(RequestType::Deposit),
                            Some("Withdrawal") => Some(RequestType::Withdrawal),
                            Some("Borrow") => Some(RequestType::Borrow),
                            _ => None,
                        };
                            
                        EventQueue::create_event(
                            EventType::ValidationFailure,
                            block_number,
                            event.transaction_hash,
                            None,
                            wallet_address,
                            amount,
                            request_type,
                            event.timestamp,
                            serde_json::to_string(&event.data).unwrap_or_default(),
                        )
                    },
                    // Add more event types as needed
                    _ => {
                        // Unknown event type, create a generic event
                        EventQueue::create_event(
                            EventType::ValidationFailure, // Default to validation failure for unknown events
                            block_number,
                            event.transaction_hash,
                            None,
                            None,
                            None,
                            None,
                            event.timestamp,
                            serde_json::to_string(&event.data).unwrap_or_default(),
                        )
                    }
                };
                
                // Enqueue the event for processing
                self.event_queue.enqueue(indexed_event).await
                    .context("Failed to enqueue event")?;
                
                event_count += 1;
            }
            
            // Update the last processed block
            self.last_processed_block = block_number;
            self.update_last_processed_block(block_number).await
                .context("Failed to update last processed block")?;
        }
        
        Ok(event_count)
    }
} 