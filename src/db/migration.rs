use anyhow::{Context, Result};
use log::info;
use sqlx::{migrate::MigrateDatabase, PgPool, Postgres};
use std::env;

/// Runs all migrations
pub async fn run_migrations(pg_pool: &PgPool) -> Result<()> {
    info!("Running migrations...");
    
    // Create schema if it doesn't exist
    sqlx::query("CREATE SCHEMA IF NOT EXISTS lsrwa_express")
        .execute(pg_pool)
        .await
        .context("Failed to create schema")?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(pg_pool)
        .await
        .context("Failed to run migrations")?;
    
    info!("Migrations completed successfully");
    
    Ok(())
}

/// Initialize the database if it doesn't exist
pub async fn ensure_database_exists() -> Result<()> {
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    
    // Extract the database name and server URL
    let parts: Vec<&str> = database_url.rsplitn(2, '/').collect();
    let (db_name, _server_url) = match parts.as_slice() {
        [name, url] => (name, format!("{}/postgres", url)),
        _ => return Err(anyhow::anyhow!("Invalid DATABASE_URL format")),
    };

    if !Postgres::database_exists(&database_url).await? {
        info!("Database '{}' does not exist, creating it", db_name);
        
        // Connect to the postgres database to create the new one
        Postgres::create_database(&database_url).await?;
        
        info!("Database '{}' created successfully", db_name);
    } else {
        info!("Database '{}' already exists", db_name);
    }
    
    Ok(())
} 