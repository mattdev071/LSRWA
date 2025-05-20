pub mod pg;
pub mod migration;

use anyhow::Result;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct DbPool {
    pub pg: PgPool,
}

impl DbPool {
    /// Create a new database pool from environment variables
    pub async fn new() -> Result<Self> {
        let pg = pg::create_pg_pool().await?;
        Ok(Self { pg })
    }

    /// Run migrations on the database
    pub async fn run_migrations(&self) -> Result<()> {
        migration::run_migrations(&self.pg).await
    }
}

/// Initialize database pool with migrations
pub async fn init_db() -> Result<DbPool> {
    let pool = DbPool::new().await?;
    pool.run_migrations().await?;
    Ok(pool)
} 