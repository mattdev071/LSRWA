use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;

pub mod migration;
pub mod pg;
pub mod kyc_repository;

/// Database pools
#[derive(Clone)]
pub struct DbPools {
    pub pg: sqlx::PgPool,
}

/// Initialize database connections
pub async fn init_db() -> Result<DbPools> {
    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    
    // Create connection pool
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .context("Failed to connect to Postgres")?;
    
    // Run migrations
    migration::run_migrations(&pg_pool).await?;
    
    Ok(DbPools {
        pg: pg_pool,
    })
} 