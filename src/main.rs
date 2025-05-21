use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use lsrwa_express_rust::api::blockchain::BlockchainState;
use lsrwa_express_rust::db;
use lsrwa_express_rust::services::BlockchainService;
// Add this line to import the indexer module
use lsrwa_express_rust::services::indexer;

// Remove local module declarations that conflict with imports
// mod api;
mod contract;
mod models;
// mod services;

// Use the API module from the crate
use lsrwa_express_rust::api;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("Starting LSRWA Express API server");
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Ensure database exists
    db::migration::ensure_database_exists().await.context("Failed to ensure database exists")?;
    
    // Initialize database connections
    let pool = db::init_db().await.context("Failed to initialize database")?;
    
    // Test connection
    db::pg::test_connection(&pool.pg).await.context("Failed to test connection")?;
    
    // Create the blockchain state
    let blockchain_state = Arc::new(RwLock::new(BlockchainState::default()));
    
    // Initialize the blockchain service
    let blockchain_service = Arc::new(
        BlockchainService::new(pool.clone(), blockchain_state.clone())
            .await
            .context("Failed to initialize blockchain service")?
    );
    
    // Create the app state
    let app_state = api::AppState {
        db: pool.clone(),
        blockchain_state: blockchain_state.clone(),
    };
    
    // Create the event indexer
    let event_processor = indexer::EventProcessor::new(
        pool.clone(),
        blockchain_service.clone(),
        blockchain_state.clone(),
        100, // buffer size
        3,   // max attempts
        300, // retry delay in seconds
        60,  // polling interval in seconds
    ).await.context("Failed to initialize event processor")?;
    
    // Start the event indexer in a separate task
    let mut event_processor_clone = event_processor;
    tokio::spawn(async move {
        tracing::info!("Starting event indexer");
        if let Err(err) = event_processor_clone.start().await {
            tracing::error!("Event indexer error: {}", err);
        }
    });
    
    // Build the API router
    let app = api::create_router(app_state)
        .layer(TraceLayer::new_for_http());
    
    // Get the port from environment or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .context("Failed to parse PORT environment variable")?;
    
    // Create the socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    tracing::info!("Listening on {}", addr);
    
    // Start the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Server error")?;
    
    Ok(())
}
