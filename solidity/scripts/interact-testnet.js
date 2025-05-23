// interact-testnet.js - Script to interact with the deployed LSRWA Express contract
const { ethers } = require("hardhat");
const fs = require("fs");

async function main() {
  // Load deployment information
  let deploymentInfo;
  try {
    const deploymentFile = "deployment-sepolia.json";
    deploymentInfo = JSON.parse(fs.readFileSync(deploymentFile, "utf8"));
    console.log(`Loaded deployment info from ${deploymentFile}`);
  } catch (error) {
    console.error("Failed to load deployment info:", error.message);
    console.log("Using hardcoded contract address instead");
    deploymentInfo = {
      contractAddress: "0xF8D02C027D917fE1B8C45214d031B9e9e18b26BC",
      network: "sepolia"
    };
  }

  // Get the network
  const network = await ethers.provider.getNetwork();
  console.log(`Connected to network: ${network.name} (chainId: ${network.chainId})`);
  
  if (network.name !== deploymentInfo.network) {
    console.warn(`Warning: Connected to ${network.name} but contract was deployed to ${deploymentInfo.network}`);
  }

  // Get the contract
  const LsrwaExpress = await ethers.getContractFactory("LsrwaExpress");
  const contract = LsrwaExpress.attach(deploymentInfo.contractAddress);
  console.log(`Connected to LSRWA Express at ${deploymentInfo.contractAddress}`);

  // Get signers
  const [owner, user1, user2] = await ethers.getSigners();
  console.log(`Using account: ${owner.address}`);

  // Read contract state
  console.log("\nReading contract state...");
  
  const ownerAddress = await contract.owner();
  console.log(`Contract owner: ${ownerAddress}`);
  
  const minDepositAmount = await contract.minDepositAmount();
  console.log(`Minimum deposit: ${ethers.utils.formatEther(minDepositAmount)} ETH`);
  
  const minWithdrawalAmount = await contract.minWithdrawalAmount();
  console.log(`Minimum withdrawal: ${ethers.utils.formatEther(minWithdrawalAmount)} ETH`);
  
  const currentEpoch = await contract.getCurrentEpoch();
  console.log(`Current epoch ID: ${currentEpoch.id}`);

  // Check if user is registered
  console.log("\nChecking if user is registered...");
  try {
    const user = await contract.getUser(owner.address);
    console.log("User info:", {
      walletAddress: user.walletAddress,
      isRegistered: user.isRegistered,
      activeBalance: ethers.utils.formatEther(user.activeBalance),
      pendingDeposits: ethers.utils.formatEther(user.pendingDeposits),
      pendingWithdrawals: ethers.utils.formatEther(user.pendingWithdrawals)
    });
    
    if (!user.isRegistered) {
      console.log("Registering user...");
      const tx = await contract.registerUser();
      await tx.wait();
      console.log("User registered successfully!");
    }
  } catch (error) {
    console.log("User not found, registering...");
    const tx = await contract.registerUser();
    await tx.wait();
    console.log("User registered successfully!");
  }

  // Make a deposit request
  console.log("\nCreating a deposit request...");
  
  try {
    const depositAmount = ethers.utils.parseEther("0.01");
    console.log(`Making a deposit request of ${ethers.utils.formatEther(depositAmount)} ETH...`);
    
    const tx = await contract.createDepositRequest(depositAmount);
    console.log(`Transaction hash: ${tx.hash}`);
    await tx.wait();
    console.log("Deposit request created!");
    
    // Check user after deposit request
    const user = await contract.getUser(owner.address);
    console.log("User after deposit request:", {
      activeBalance: ethers.utils.formatEther(user.activeBalance),
      pendingDeposits: ethers.utils.formatEther(user.pendingDeposits)
    });
    
    // Get user deposit requests
    const depositRequests = await contract.getUserDepositRequests(owner.address);
    console.log(`User has ${depositRequests.length} deposit requests`);
    
    if (depositRequests.length > 0) {
      const latestRequestId = depositRequests[depositRequests.length - 1];
      console.log(`Latest request ID: ${latestRequestId}`);
      
      const request = await contract.getRequest(latestRequestId);
      console.log("Latest request:", {
        id: request.id.toString(),
        requestType: request.requestType,
        walletAddress: request.walletAddress,
        amount: ethers.utils.formatEther(request.amount),
        timestamp: new Date(request.timestamp.toNumber() * 1000).toISOString(),
        isProcessed: request.isProcessed
      });
    }
  } catch (error) {
    console.error("Failed to create deposit request:", error.message);
  }

  // Check contract balance
  const contractBalance = await contract.getContractBalance();
  console.log(`\nContract balance: ${ethers.utils.formatEther(contractBalance)} ETH`);
}

// Execute the script
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("Script failed:", error);
    process.exit(1);
  }); 