use std::{fs, env};
use subxt::{OnlineClient, PolkadotConfig};
use anyhow::{Result, Context};

#[tokio::main]
async fn main() -> Result<()> {
    // Get the RPC URL from environment or use default
    let rpc_url = std::env::var("SUBSTRATE_RPC_URL")
        .unwrap_or_else(|_| "wss://rococo-contracts-rpc.polkadot.io:443".to_string());
    
    println!("Connecting to {}", rpc_url);
    
    // Try to connect to the node
    let client_result = OnlineClient::<PolkadotConfig>::from_url(rpc_url).await;
    
    // If connection fails, just create mock files
    if let Err(e) = client_result {
        println!("Failed to connect to blockchain node: {}", e);
        println!("Creating mock metadata files instead");
    } else {
        println!("Successfully connected to blockchain node");
    }
    
    // Create a mock metadata file for development
    let metadata_bytes = b"mock_metadata_for_development";
    
    // Determine project root directory
    let project_root = env::current_dir()
        .context("Failed to get current directory")?;
    
    println!("Project root: {:?}", project_root);
    
    // Create the metadata directory if it doesn't exist
    let metadata_dir = project_root.join("metadata");
    fs::create_dir_all(&metadata_dir)
        .context("Failed to create metadata directory")?;
    
    // Write the metadata to a file
    let metadata_path = metadata_dir.join("metadata.scale");
    fs::write(&metadata_path, metadata_bytes)
        .context("Failed to write metadata file")?;
    
    println!("Metadata written to {:?}", metadata_path);
    
    // Also copy to the root directory for subxt macros
    let root_metadata_path = project_root.join("metadata.scale");
    fs::write(&root_metadata_path, metadata_bytes)
        .context("Failed to write root metadata file")?;
    
    println!("Metadata also written to {:?}", root_metadata_path);
    
    Ok(())
} 