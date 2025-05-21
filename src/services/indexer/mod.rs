//! Event indexer service for LSRWA Express
//! 
//! This module provides functionality to index and queue on-chain events from the LSRWA Express contract.

mod event_processor;
mod event_queue;
mod event_types;

pub use event_processor::EventProcessor;
pub use event_queue::EventQueue;
pub use event_types::{EventType, IndexedEvent, ProcessingStatus};
