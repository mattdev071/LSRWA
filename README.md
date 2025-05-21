# LSRWA Express Rust

## Production-Ready Contract Deployment and Integration Guide

This guide will walk you through deploying the LSRWA Express smart contract to a production or testnet environment and integrating with it from the backend.

### Prerequisites

- Rust toolchain with `cargo` installed
- ink! contract toolchain (`cargo install cargo-contract`)
- Access to a Substrate node with contracts pallet enabled
- An account with sufficient tokens for deployment

### Environment Setup

Set the following environment variables:

```bash
# The RPC URL of the node
export SUBSTRATE_RPC_URL="wss://rococo-contracts-rpc.polkadot.io"

# Seed phrase for your wallet (keep this secure!)
export WALLET_SEED_PHRASE="your mnemonic seed phrase here"

# Optional: Contract deployment configuration
export CONTRACT_WASM_PATH="target/ink/lsrwa_express.wasm"
export CONTRACT_METADATA_PATH="target/ink/metadata.json"
export CONTRACT_CONSTRUCTOR="new"
export CONTRACT_GAS_LIMIT="500000000000"
export CONTRACT_VALUE="0"
```

### Project Structure

The project is organized with proper separation of concerns:

- `contracts/` - Smart contract code written in ink!
- `src/contract/` - Contract bindings and interaction helpers
- `src/services/` - Backend services including blockchain integration
- `scripts/` - Deployment and metadata tools
- `metadata/` - Chain metadata storage

### Building the Contract

```bash
# Build the contract
cargo contract build --release
```

This will generate the WASM binary and metadata in `target/ink/`.

### Downloading Chain Metadata

Before interacting with the blockchain, we need to download the chain metadata:

```bash
# Run the metadata download script
cargo run --bin download_metadata
```

This will create metadata files in both the `metadata/` directory and project root, which our code uses for type information.

### Deploying the Contract

```bash
# Run the production-ready deployment script
cargo run --bin deploy_contract
```

This script will:
1. Upload the contract WASM to the blockchain
2. Instantiate the contract with proper gas estimation
3. Save deployment information to `deployment_info.json`
4. Create a `.env.contract` file with the contract address
5. Output the contract address and transaction details

After deployment, the script will automatically set up the necessary configuration files, but you can also manually set the `CONTRACT_ADDRESS` environment variable:

```bash
export CONTRACT_ADDRESS="contract_address_from_deployment"
```

### Contract Interaction Architecture

The backend uses a production-ready architecture for contract interaction:

1. **Type-Safe Contract Bindings**: Generated during build process
2. **Gas Estimation**: Dynamic gas estimation based on call complexity
3. **Error Handling**: Proper error propagation and context
4. **Transaction Monitoring**: Complete transaction lifecycle management
5. **Event Parsing**: Structured event parsing for contract events

### Making Deposit Requests

With the contract deployed, you can make deposit requests through the API:

```bash
curl -X POST http://localhost:3000/api/v1/requests/deposit \
  -H "Content-Type: application/json" \
  -d '{"wallet_address": "your_wallet_address", "amount": 100}'
```

### Security Considerations

- **Key Management**: In production, use a proper key management system (AWS KMS, HashiCorp Vault, etc.)
- **Error Handling**: All blockchain interactions include proper error handling and logging
- **Gas Estimation**: Dynamic gas estimation prevents transaction failures
- **Transaction Monitoring**: All transactions are monitored for finalization
- **Environment Variables**: Sensitive configuration is managed via environment variables

### Monitoring and Maintenance

- **Deployment Info**: All deployments are recorded in `deployment_info.json`
- **Transaction Records**: All transactions are stored in the database with block numbers and hashes
- **Logging**: Comprehensive logging of all blockchain interactions

### Troubleshooting

- **Connection Issues**: Check the RPC URL and network connectivity
- **Transaction Failures**: Check gas limits and account balances
- **Contract Errors**: Check the contract logs for specific error messages

### Production Deployment Checklist

- [ ] Secure key management solution configured
- [ ] Environment variables set in production environment
- [ ] Database migrations applied
- [ ] Contract deployed to production network
- [ ] Gas limits properly configured
- [ ] Error monitoring and alerting set up
- [ ] Load testing completed
- [ ] Security audit performed 