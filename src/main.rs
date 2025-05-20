use anyhow::{Context, Result};
use lsrwa_express_rust::db;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    println!("Initializing database and running migrations...");

    // Ensure database exists
    db::migration::ensure_database_exists().await.context("Failed to ensure database exists")?;

    // Initialize database and run migrations
    let pool = db::init_db().await.context("Failed to initialize database")?;

    // Test connection
    db::pg::test_connection(&pool.pg).await.context("Failed to test connection")?;

    println!("Database setup completed successfully!");

    Ok(())
}
