use anyhow::{Context, Result};
use log::{info, warn};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use std::time::Duration;

/// Create a PostgreSQL connection pool from environment variables
pub async fn create_pg_pool() -> Result<PgPool> {
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    
    let max_connections = env::var("PG_MAX_CONNECTIONS")
        .unwrap_or_else(|_| {
            warn!("PG_MAX_CONNECTIONS not set, using default value of 5");
            "5".to_string()
        })
        .parse::<u32>()
        .context("PG_MAX_CONNECTIONS must be a number")?;

    info!("Connecting to PostgreSQL with up to {} connections", max_connections);
    
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
        .context("Failed to create PostgreSQL connection pool")?;
    
    info!("Connected to PostgreSQL database");
    
    Ok(pool)
}

/// Test the database connection
pub async fn test_connection(pool: &PgPool) -> Result<()> {
    let result = sqlx::query!("SELECT NOW() as time")
        .fetch_one(pool)
        .await
        .context("Failed to query database")?;
    
    info!("Database connection successful, server time: {:?}", result.time);
    
    Ok(())
} 