use std::{fs, path::Path, env};
use anyhow::{Result, Context, anyhow};
use subxt::{
    OnlineClient, 
    PolkadotConfig, 
    tx::PairSigner,
    utils::MultiAddress,
    ext::sp_core::{sr25519, Pair as PairTrait}
};
use ink::env::Environment;
use scale::{Encode, Decode};

// Import the runtime API that we generated from metadata
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod substrate {}

// Contract deployment configuration
struct DeploymentConfig {
    rpc_url: String,
    seed_phrase: String,
    contract_path: String,
    metadata_path: String,
    constructor_name: String,
    constructor_args: Vec<String>,
    gas_limit: u64,
    value: u128,
}

impl DeploymentConfig {
    fn from_env() -> Result<Self> {
        // Get the RPC URL
        let rpc_url = std::env::var("SUBSTRATE_RPC_URL")
            .unwrap_or_else(|_| "wss://rococo-contracts-rpc.polkadot.io".to_string());
        
        // Get the seed phrase
        let seed_phrase = std::env::var("WALLET_SEED_PHRASE")
            .context("WALLET_SEED_PHRASE environment variable must be set")?;
        
        // Get the contract path
        let contract_path = std::env::var("CONTRACT_WASM_PATH")
            .unwrap_or_else(|_| "target/ink/lsrwa_express.wasm".to_string());
            
        // Get the metadata path
        let metadata_path = std::env::var("CONTRACT_METADATA_PATH")
            .unwrap_or_else(|_| "target/ink/metadata.json".to_string());
            
        // Get the constructor name
        let constructor_name = std::env::var("CONTRACT_CONSTRUCTOR")
            .unwrap_or_else(|_| "new".to_string());
            
        // Get constructor arguments (none for our constructor)
        let constructor_args = Vec::new();
        
        // Get gas limit
        let gas_limit = std::env::var("CONTRACT_GAS_LIMIT")
            .unwrap_or_else(|_| "500000000000".to_string())
            .parse()
            .context("Invalid CONTRACT_GAS_LIMIT")?;
            
        // Get value to send with deployment
        let value = std::env::var("CONTRACT_VALUE")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .context("Invalid CONTRACT_VALUE")?;
            
        Ok(Self {
            rpc_url,
            seed_phrase,
            contract_path,
            metadata_path,
            constructor_name,
            constructor_args,
            gas_limit,
            value,
        })
    }
}

// Contract deployment result
struct DeploymentResult {
    code_hash: [u8; 32],
    contract_address: String,
    transaction_hash: String,
    block_number: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Get deployment configuration
    let config = DeploymentConfig::from_env()
        .context("Failed to get deployment configuration")?;
    
    println!("Connecting to {}", config.rpc_url);
    
    // Connect to the node
    let client = OnlineClient::<PolkadotConfig>::from_url(config.rpc_url)
        .await
        .context("Failed to connect to blockchain node")?;
    
    // Create account from seed phrase
    let pair = sr25519::Pair::from_string(&config.seed_phrase, None)
        .map_err(|_| anyhow!("Invalid seed phrase"))?;
    
    let signer = PairSigner::new(pair);
    let account = signer.account_id().clone();
    
    println!("Deploying contract from account: {}", account);
    
    // Read the contract WASM binary
    let contract_path = Path::new(&config.contract_path);
    let wasm_code = fs::read(contract_path)
        .context(format!("Failed to read contract WASM from {:?}", contract_path))?;
    
    println!("Contract WASM size: {} bytes", wasm_code.len());
    
    // Read the contract metadata
    let metadata_path = Path::new(&config.metadata_path);
    let metadata_str = fs::read_to_string(metadata_path)
        .context(format!("Failed to read contract metadata from {:?}", metadata_path))?;
    
    // Parse the metadata
    let metadata: serde_json::Value = serde_json::from_str(&metadata_str)
        .context("Failed to parse contract metadata")?;
    
    // Find the constructor
    let constructors = metadata["spec"]["constructors"].as_array()
        .ok_or_else(|| anyhow!("Invalid metadata format: constructors not found"))?;
        
    let constructor = constructors.iter()
        .find(|c| c["name"].as_str().unwrap_or("") == config.constructor_name)
        .ok_or_else(|| anyhow!("Constructor {} not found", config.constructor_name))?;
    
    println!("Using constructor: {}", config.constructor_name);
    
    // Encode constructor arguments
    let constructor_data = encode_constructor_data(&config.constructor_name, &config.constructor_args)?;
    
    // Upload the code
    println!("Uploading contract code...");
    
    use substrate::tx::contracts::upload_code;
    
    let upload_tx = upload_code {
        code: wasm_code,
        storage_deposit_limit: None,
        determinism: substrate::runtime_types::pallet_contracts::wasm::Determinism::Enforced,
    };
    
    let tx = client
        .tx()
        .create_signed(&upload_tx, &signer, Default::default())
        .await
        .context("Failed to create signed transaction for code upload")?;
    
    let events = tx.submit_and_watch()
        .await
        .context("Failed to submit code upload transaction")?
        .wait_for_finalized_success()
        .await
        .context("Code upload transaction failed or was not included in a block")?;
    
    // Find the code hash from the events
    let code_stored_event = events.find_first::<substrate::contracts::events::CodeStored>()
        .context("Failed to find CodeStored event")?;
    
    if let Some(event) = code_stored_event {
        let code_hash = event.code_hash;
        println!("Contract code uploaded with hash: 0x{}", hex::encode(code_hash));
        
        // Instantiate the contract
        println!("Instantiating contract...");
        
        use substrate::tx::contracts::instantiate;
        
        let instantiate_tx = instantiate {
            code_hash,
            gas_limit: config.gas_limit,
            storage_deposit_limit: None,
            data: constructor_data,
            salt: Vec::new(),
            value: config.value,
        };
        
        let tx = client
            .tx()
            .create_signed(&instantiate_tx, &signer, Default::default())
            .await
            .context("Failed to create signed transaction for instantiation")?;
        
        let events = tx.submit_and_watch()
            .await
            .context("Failed to submit instantiation transaction")?
            .wait_for_finalized_success()
            .await
            .context("Instantiation transaction failed or was not included in a block")?;
        
        // Find the instantiation event
        let instantiated_event = events.find_first::<substrate::contracts::events::Instantiated>()
            .context("Failed to find Instantiated event")?;
        
        if let Some(event) = instantiated_event {
            let contract_address = event.contract;
            let tx_hash = events.extrinsic_hash();
            let block_hash = events.block_hash();
            
            // Get block number
            let block = client.blocks().at(block_hash)
                .await
                .context("Failed to get block")?;
            
            let block_number = block.header().number;
            
            // Create deployment result
            let result = DeploymentResult {
                code_hash: code_hash.into(),
                contract_address: format!("{:?}", contract_address),
                transaction_hash: format!("0x{}", hex::encode(tx_hash.as_ref())),
                block_number,
            };
            
            println!("\nContract successfully deployed!");
            println!("Contract address: {}", result.contract_address);
            println!("Code hash: 0x{}", hex::encode(result.code_hash));
            println!("Transaction hash: {}", result.transaction_hash);
            println!("Block number: {}", result.block_number);
            
            // Save deployment info to file
            let deployment_info = serde_json::json!({
                "contract_address": result.contract_address,
                "code_hash": format!("0x{}", hex::encode(result.code_hash)),
                "transaction_hash": result.transaction_hash,
                "block_number": result.block_number,
                "deployed_at": chrono::Utc::now().to_rfc3339(),
            });
            
            let deployment_info_path = Path::new("deployment_info.json");
            fs::write(
                deployment_info_path, 
                serde_json::to_string_pretty(&deployment_info).unwrap()
            ).context("Failed to write deployment info")?;
            
            println!("\nDeployment info saved to {:?}", deployment_info_path);
            println!("\nSet the following environment variable to use this contract:");
            println!("export CONTRACT_ADDRESS={}", result.contract_address);
            
            // Create .env file with contract address
            let env_content = format!("CONTRACT_ADDRESS={}\n", result.contract_address);
            fs::write(".env.contract", env_content)
                .context("Failed to write .env.contract file")?;
                
            println!("Also saved to .env.contract");
        } else {
            return Err(anyhow!("Failed to find instantiation event"));
        }
    } else {
        return Err(anyhow!("Failed to find code storage event"));
    }
    
    Ok(())
}

// Helper function to encode constructor data
fn encode_constructor_data(constructor_name: &str, args: &[String]) -> Result<Vec<u8>> {
    // For our simple case with no arguments, we just need the selector
    let selector = get_selector(constructor_name);
    
    // In a real implementation, we would encode the arguments here
    // For now, we just return the selector
    Ok(selector.to_vec())
}

// Helper function to get a function selector
fn get_selector(function_name: &str) -> [u8; 4] {
    use ink::env::hash::{Blake2x256, HashOutput};
    
    // Compute the selector - first 4 bytes of the Blake2 hash of the function signature
    let mut output = <<Blake2x256 as HashOutput>::Type as Default>::default();
    ink::env::hash_bytes::<Blake2x256>(function_name.as_bytes(), &mut output);
    
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&output[0..4]);
    
    selector
} 