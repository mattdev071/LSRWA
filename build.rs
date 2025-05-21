use std::process::Command;
use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=contracts/lib.rs");
    println!("cargo:rerun-if-changed=contracts/Cargo.toml");
    println!("cargo:rerun-if-changed=contracts/Cargo.lock");
    
    // Check if we're building for the host platform (not wasm32)
    let target = env::var("TARGET").unwrap_or_default();
    
    if !target.contains("wasm32") {
        println!("cargo:warning=Skipping contract compilation for non-wasm32 target");
        // Still generate the bindings, but with placeholder code
        generate_placeholder_bindings()?;
    } else {
        // Generate contract bindings for wasm32 target
        generate_contract_bindings()?;
    }
    
    Ok(())
}

fn generate_placeholder_bindings() -> Result<(), Box<dyn std::error::Error>> {
    // Create the directory for generated code
    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_dir = Path::new(&out_dir).join("generated");
    fs::create_dir_all(&generated_dir)?;
    
    // Generate placeholder bindings that will compile on the host platform
    let binding_code = r#"
// Placeholder contract bindings for non-wasm32 targets
use std::fmt;

// Dummy types to make the code compile
pub type AccountId = [u8; 32];
pub type H256 = [u8; 32];

// Contract interface
pub struct LsrwaExpressContract {
    pub client: (),
    pub address: AccountId,
}

// Selector for create_deposit_request
pub const CREATE_DEPOSIT_REQUEST_SELECTOR: [u8; 4] = [0x44, 0x79, 0x78, 0x8a];

// Result types
#[derive(Debug)]
pub enum DepositRequestResult {
    Ok(u128),
    Err(()),
}

impl fmt::Display for DepositRequestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DepositRequestResult::Ok(id) => write!(f, "Request ID: {}", id),
            DepositRequestResult::Err(_) => write!(f, "Error"),
        }
    }
}

impl LsrwaExpressContract {
    pub fn new(_client: (), address: AccountId) -> Self {
        Self { client: (), address }
    }
    
    // Create deposit request method (placeholder)
    pub async fn create_deposit_request(
        &self, 
        _signer: &(),
        _amount: u128,
        _gas_limit: u64,
    ) -> Result<H256, Box<dyn std::error::Error>> {
        // This is just a placeholder that will compile but not be used
        Err("Contract calls not available in non-wasm32 builds".into())
    }
}
"#;

    let binding_path = generated_dir.join("contract_bindings.rs");
    fs::write(binding_path, binding_code)?;
    
    Ok(())
}

fn generate_contract_bindings() -> Result<(), Box<dyn std::error::Error>> {
    // Create the directory for generated code
    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_dir = Path::new(&out_dir).join("generated");
    fs::create_dir_all(&generated_dir)?;
    
    // Ensure the contract is built
    let status = Command::new("cargo")
        .args(&["contract", "build", "--release", "--manifest-path", "contracts/Cargo.toml"])
        .status()?;
        
    if !status.success() {
        return Err("Failed to build contract".into());
    }
    
    // Copy metadata.json to the generated directory
    let metadata_source = Path::new("target/ink/metadata.json");
    let metadata_dest = generated_dir.join("metadata.json");
    fs::copy(metadata_source, metadata_dest)?;
    
    // Generate bindings using subxt-codegen or a similar tool
    // Here we'll create a simple binding generator module for demonstration
    let binding_code = r#"
// Auto-generated contract bindings for LSRWA Express
use ink::primitives::AccountId;
use ink::env::DefaultEnvironment;
use ink::LangError;
use scale::{Encode, Decode};
use subxt::{
    tx::PairSigner,
    utils::MultiAddress,
    OnlineClient,
    PolkadotConfig,
    ext::sp_core::{sr25519, ByteArray, Pair as PairTrait, H256}
};
use std::fmt;

// Contract interface
pub struct LsrwaExpressContract {
    pub client: OnlineClient<PolkadotConfig>,
    pub address: AccountId,
}

// Selector for create_deposit_request
pub const CREATE_DEPOSIT_REQUEST_SELECTOR: [u8; 4] = [0x44, 0x79, 0x78, 0x8a];

// Result types
#[derive(Debug, Encode, Decode)]
pub enum DepositRequestResult {
    Ok(u128),
    Err(LangError),
}

impl fmt::Display for DepositRequestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DepositRequestResult::Ok(id) => write!(f, "Request ID: {}", id),
            DepositRequestResult::Err(err) => write!(f, "Error: {:?}", err),
        }
    }
}

impl LsrwaExpressContract {
    pub fn new(client: OnlineClient<PolkadotConfig>, address: AccountId) -> Self {
        Self { client, address }
    }
    
    // Create deposit request method
    pub async fn create_deposit_request(
        &self, 
        signer: &PairSigner<PolkadotConfig, sr25519::Pair>,
        amount: u128,
        gas_limit: u64,
    ) -> Result<H256, Box<dyn std::error::Error>> {
        use subxt::tx::SubmittableExtrinsic;
        
        // Prepare the call data - selector + encoded parameters
        let mut call_data = CREATE_DEPOSIT_REQUEST_SELECTOR.to_vec();
        
        // Encode the amount parameter (SCALE encoding)
        let mut amount_bytes = Vec::new();
        amount.encode_to(&mut amount_bytes);
        call_data.extend(amount_bytes);
        
        // Contract call
        use crate::substrate::tx::contracts::call;
        
        // Value to send with the call (0 for now)
        let value = 0u128;
        
        // Call parameters
        let params = call {
            dest: MultiAddress::Id(self.address.into()),
            value,
            gas_limit,
            storage_deposit_limit: None,
            data: call_data,
        };
        
        // Create the signed transaction
        let tx = self.client
            .tx()
            .create_signed(&params, signer, Default::default())
            .await?;
            
        // Submit and watch for finalization
        let events = tx.submit_and_watch()
            .await?
            .wait_for_finalized_success()
            .await?;
            
        // Get the transaction hash
        let tx_hash = events.extrinsic_hash();
        
        Ok(tx_hash)
    }
}
"#;

    let binding_path = generated_dir.join("contract_bindings.rs");
    fs::write(binding_path, binding_code)?;
    
    Ok(())
} 