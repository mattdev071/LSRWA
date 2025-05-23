# LSRWA Express Solidity Implementation

This directory contains the Solidity implementation of the LSRWA Express contract for deployment on EVM-compatible blockchains.

## Overview

The LSRWA Express contract implements a hybrid on-chain/off-chain request system for handling deposits, withdrawals, and borrowing with the following features:

- User registration and management
- Deposit request handling
- Withdrawal request processing
- Borrow request handling with collateral validation
- Epoch-based batch processing
- Emergency withdrawal functionality

## Contract Structure

The contract is organized into several logical sections:

1. **Type Definitions**: Enums and structs for requests, users, and epochs
2. **State Variables**: Contract storage including mappings and configuration
3. **Events**: Event definitions for all state changes
4. **Modifiers**: Access control and validation modifiers
5. **User Management**: Functions for user registration and queries
6. **Request Management**: Functions for creating and querying requests
7. **Request Processing**: Functions for processing individual requests
8. **Batch Processing**: Functions for processing multiple requests efficiently
9. **Epoch Management**: Functions for managing processing epochs
10. **Withdrawal Execution**: Functions for executing withdrawals
11. **Query Functions**: Helper functions for retrieving data

## Setup and Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd lsrwa-express-rust/contracts/solidity
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

3. **Configure environment variables**:
   Create a `.env` file with the following variables:
   ```
   PRIVATE_KEY=your_private_key_here
   ALCHEMY_API_KEY=your_alchemy_api_key_here
   ETHERSCAN_API_KEY=your_etherscan_api_key_here
   INFURA_API_KEY=your_infura_api_key_here
   VERIFY_CONTRACT=true
   ```

## Deployment

### Deploying to a Testnet

The contract can be deployed to various testnets (Sepolia, Goerli, Mumbai, etc.) using the provided scripts:

1. **Compile the contract**:
   ```bash
   npx hardhat compile
   ```

2. **Deploy to Sepolia testnet**:
   ```bash
   npx hardhat run scripts/deploy-testnet.js --network sepolia
   ```

3. **Deploy to Mumbai testnet**:
   ```bash
   npx hardhat run scripts/deploy-testnet.js --network mumbai
   ```

### Deployment Output

Upon successful deployment, you'll see output similar to:

```
Deploying LSRWA Express contract to testnet...
Deploying to network: sepolia (chainId: 11155111)
Deploying contract...
Waiting for deployment transaction to be mined...
LSRWA Express contract deployed to: 0xF8D02C027D917fE1B8C45214d031B9e9e18b26BC
Waiting for block confirmations...
Contract verified successfully

Deployment Information:
------------------------
Network: sepolia
Contract Address: 0xF8D02C027D917fE1B8C45214d031B9e9e18b26BC
Transaction Hash: 0xe07c0c29e9de7f917834f2c3de2b1d7441e6468d6c3a85007545c344ad5ced9f
Deployer Address: 0xF6927DF9199913eA65EB552306067d76d1DD2A64
```

The contract address will be needed for interacting with the deployed contract.

## Interacting with the Contract

### Using the Interaction Script

The project includes a script for interacting with the deployed contract:

```bash
npx hardhat run scripts/interact-testnet.js --network sepolia
```

This script performs the following actions:
1. Connects to the deployed contract
2. Reads contract state (owner, minimum deposit/withdrawal amounts, current epoch)
3. Registers the user if not already registered
4. Creates a deposit request
5. Retrieves and displays user information and request details

### Key Interaction Functions

#### User Registration
```javascript
const tx = await contract.registerUser();
await tx.wait();
```

#### Creating a Deposit Request
```javascript
const depositAmount = ethers.utils.parseEther("0.01");
const tx = await contract.createDepositRequest(depositAmount);
await tx.wait();
```

#### Creating a Withdrawal Request
```javascript
const withdrawAmount = ethers.utils.parseEther("0.005");
const tx = await contract.createWithdrawalRequest(withdrawAmount);
await tx.wait();
```

#### Checking User Information
```javascript
const user = await contract.getUser(address);
console.log("User info:", {
  walletAddress: user.walletAddress,
  isRegistered: user.isRegistered,
  activeBalance: ethers.utils.formatEther(user.activeBalance),
  pendingDeposits: ethers.utils.formatEther(user.pendingDeposits),
  pendingWithdrawals: ethers.utils.formatEther(user.pendingWithdrawals)
});
```

#### Retrieving Request Information
```javascript
const request = await contract.getRequest(requestId);
console.log("Request:", {
  id: request.id.toString(),
  requestType: request.requestType,
  walletAddress: request.walletAddress,
  amount: ethers.utils.formatEther(request.amount),
  timestamp: new Date(request.timestamp.toNumber() * 1000).toISOString(),
  isProcessed: request.isProcessed
});
```

### Using Etherscan

You can also interact with the deployed contract through Etherscan:

1. Navigate to the contract on Etherscan (e.g., https://sepolia.etherscan.io/address/0xF8D02C027D917fE1B8C45214d031B9e9e18b26BC)
2. Go to the "Contract" tab
3. Click on "Write Contract" to send transactions
4. Click on "Read Contract" to view contract state

## Integration with Off-Chain Components

This contract is designed to work with off-chain components:
- User Portal for KYC verification and user interface
- Request Indexing Service for monitoring on-chain events
- Epoch Processor for preparing batch updates
- Admin Dashboard for system management

KYC verification is intended to be handled through Swipelux integration in the off-chain components.

## Differences from Rust/ink! Implementation

The Solidity implementation maintains the same core functionality as the ink! implementation but with the following adaptations:

1. Uses Solidity-specific syntax and patterns
2. Leverages EVM features like `try/catch` for batch processing
3. Uses `address` type instead of `AccountId`
4. Implements native ETH handling for withdrawals
5. Uses Solidity modifiers for access control

## Gas Optimization

The contract includes several gas optimizations:
- Batch processing to reduce transaction costs
- Efficient storage layout
- Minimal use of storage for tracking requests
- Use of appropriate data types (uint128, uint32) to reduce storage costs

## Security Considerations

The contract implements several security features:
- Owner-only access for administrative functions
- Proper validation of request states and balances
- Checks for sufficient balances before transfers
- Emergency withdrawal functionality for the contract owner 