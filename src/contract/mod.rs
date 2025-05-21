//! Contract interface module for LSRWA Express

// Include the generated contract bindings
include!(concat!(env!("OUT_DIR"), "/generated/contract_bindings.rs"));

// Import the substrate module
#[cfg(not(target_arch = "wasm32"))]
pub use subxt::config::substrate;

// A simple gas estimator
pub fn estimate_gas_for_deposit_request(amount: u128) -> u64 {
    // In a production environment, this would use the dry-run API to estimate gas
    // For now, we'll use a base amount plus some scaling with the input size
    let base_gas: u64 = 5_000_000_000;
    
    // More complex logic could be added to account for the complexity of operations
    // For this example, we'll just add more gas for larger amounts (more digits)
    let amount_digits = if amount == 0 { 1 } else { (amount as f64).log10() as u64 + 1 };
    
    // Adjust gas based on input size
    base_gas + (amount_digits * 100_000_000)
}

// Helper to create the contract interface with proper configuration
#[cfg(not(target_arch = "wasm32"))]
pub async fn create_contract_interface(
    _client: subxt::OnlineClient<subxt::PolkadotConfig>,
    contract_address: &str,
) -> Result<LsrwaExpressContract, Box<dyn std::error::Error>> {
    use subxt::utils::AccountId32;
    use std::str::FromStr;
    
    // Parse the contract address
    let account_id = AccountId32::from_str(contract_address)?;
    let mut address = [0u8; 32];
    address.copy_from_slice(account_id.as_ref());
    
    // For non-wasm32 targets, we use a placeholder client
    Ok(LsrwaExpressContract::new((), address))
}

// For wasm32 target, use the original implementation
#[cfg(target_arch = "wasm32")]
pub async fn create_contract_interface(
    client: subxt::OnlineClient<subxt::PolkadotConfig>,
    contract_address: &str,
) -> Result<LsrwaExpressContract, Box<dyn std::error::Error>> {
    // Parse the contract address
    let address = ink::primitives::AccountId::try_from(
        subxt::utils::AccountId32::from_str(contract_address)?.as_ref()
    )?;
    
    // Create the contract interface
    Ok(LsrwaExpressContract::new(client, address))
}

// Helper function to parse Substrate events for deposit request results
pub fn parse_deposit_request_result(_events: &subxt::events::Events<subxt::PolkadotConfig>) -> Option<u128> {
    // In a full implementation, we would search for the contract event in the events
    // For now, just return None as a placeholder
    None
} 