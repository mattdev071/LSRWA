use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use lsrwa_express_rust::db;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    println!("=== LSRWA Express Database Schema Test ===");
    
    // Ensure database exists
    db::migration::ensure_database_exists().await.context("Failed to ensure database exists")?;
    
    println!("✅ Database exists or was created");
    
    // Get database connection pool
    let pool = db::DbPool::new().await.context("Failed to create database pool")?;
    
    // Explicitly run migrations
    pool.run_migrations().await.context("Failed to run migrations")?;
    
    println!("✅ Database migrations applied successfully");
    
    // Test connection
    db::pg::test_connection(&pool.pg).await.context("Failed to test connection")?;
    
    println!("✅ Database connection successful");
    
    // Insert test data
    insert_test_data(&pool.pg).await.context("Failed to insert test data")?;
    
    println!("✅ Test data inserted successfully");
    
    // Query and validate test data
    validate_test_data(&pool.pg).await.context("Failed to validate test data")?;
    
    println!("✅ Test data validation successful");
    
    println!("All tests passed successfully!");
    
    Ok(())
}

async fn insert_test_data(pool: &PgPool) -> Result<()> {
    // Create a test user
    let user_id = sqlx::query_as::<_, (uuid::Uuid,)>(
        r#"
        INSERT INTO lsrwa_express.users (wallet_address, email, kyc_status)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind("0x1234567890123456789012345678901234567890")
    .bind("test@example.com")
    .bind("approved")
    .fetch_one(pool)
    .await
    .context("Failed to insert test user")?
    .0;
    
    println!("📝 Created test user with ID: {}", user_id);
    
    // Create user balance
    sqlx::query(
        r#"
        INSERT INTO lsrwa_express.user_balances (user_id, active_balance, total_deposited)
        VALUES ($1, $2::numeric, $3::numeric)
        "#,
    )
    .bind(user_id)
    .bind("1000.0")
    .bind("1000.0")
    .execute(pool)
    .await
    .context("Failed to insert user balance")?;
    
    // Create an epoch
    let epoch_id = sqlx::query_as::<_, (i32,)>(
        r#"
        SELECT lsrwa_express.create_new_epoch() AS id
        "#,
    )
    .fetch_one(pool)
    .await
    .context("Failed to create epoch")?
    .0;
    
    println!("📝 Created test epoch with ID: {}", epoch_id);
    
    // Get current timestamp
    let now = Utc::now().naive_utc();
    
    // Record an on-chain deposit request
    let request_id = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT lsrwa_express.record_blockchain_request(
            'deposit',
            1,
            $1,
            $2::numeric,
            NULL,
            $3,
            12345678,
            '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
        ) AS id
        "#,
    )
    .bind("0x1234567890123456789012345678901234567890")
    .bind("500.0")
    .bind(now)
    .fetch_one(pool)
    .await
    .context("Failed to record blockchain request")?
    .0;
    
    println!("📝 Created test blockchain request with ID: {}", request_id);
    
    // Record batch processing
    let processing_id = sqlx::query_as::<_, (i32,)>(
        r#"
        SELECT lsrwa_express.record_batch_processing(
            $1,
            'deposit',
            ARRAY[1]::bigint[],
            '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
            12345680,
            $2
        ) AS id
        "#,
    )
    .bind(epoch_id)
    .bind(now)
    .fetch_one(pool)
    .await
    .context("Failed to record batch processing")?
    .0;
    
    println!("📝 Created test batch processing event with ID: {}", processing_id);
    
    Ok(())
}

async fn validate_test_data(pool: &PgPool) -> Result<()> {
    // Verify that the user exists
    let user = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT wallet_address, kyc_status
        FROM lsrwa_express.users
        WHERE email = $1
        "#,
    )
    .bind("test@example.com")
    .fetch_one(pool)
    .await
    .context("Failed to fetch test user")?;
    
    println!("🔍 Found user with wallet: {}, KYC: {}", user.0, user.1);
    
    // Verify that the blockchain request exists and was processed
    let request = sqlx::query_as::<_, (String, i64, bool)>(
        r#"
        SELECT request_type, on_chain_id, is_processed
        FROM lsrwa_express.blockchain_requests
        WHERE on_chain_id = 1 AND request_type = 'deposit'
        "#,
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch blockchain request")?;
    
    println!("🔍 Found {} request with ID: {}, processed: {}", 
             request.0, request.1, request.2);
    
    // Check that processing event exists
    let processing = sqlx::query_as::<_, (String, i32)>(
        r#"
        SELECT processing_type, processed_count
        FROM lsrwa_express.request_processing_events
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch processing event")?;
    
    println!("🔍 Found processing event of type: {}, count: {}", 
             processing.0, processing.1);
    
    // Check that batch item exists
    let batch_item = sqlx::query_as::<_, (i64, String)>(
        r#"
        SELECT request_id, status
        FROM lsrwa_express.batch_processing_items
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch batch item")?;
    
    println!("🔍 Found batch item for request ID: {}, status: {}", 
             batch_item.0, batch_item.1);
    
    Ok(())
} 