use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use lsrwa_express_rust::api::{self, blockchain::BlockchainState};
use lsrwa_express_rust::db;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("Initializing database and running migrations...");

    // Ensure database exists
    db::migration::ensure_database_exists().await.context("Failed to ensure database exists")?;

    // Initialize database and run migrations
    let pool = db::init_db().await.context("Failed to initialize database")?;

    // Test connection
    db::pg::test_connection(&pool.pg).await.context("Failed to test connection")?;

    tracing::info!("Database setup completed successfully!");
    
    // Initialize blockchain state
    let blockchain_state = Arc::new(RwLock::new(BlockchainState::default()));
    
    // Create app state
    let app_state = api::AppState {
        db: pool,
        blockchain_state,
    };
    
    // Build the API router
    let app = api::create_router(app_state)
        .layer(TraceLayer::new_for_http());
    
    // Get the port from environment or use a default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .context("Failed to parse PORT")?;
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);
    
    // Start the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Server error")?;

    Ok(())
}
